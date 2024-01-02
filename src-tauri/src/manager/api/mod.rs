pub(crate) mod delete;
pub(crate) mod list;
pub(crate) mod upload;

use std::{path::Path, str::FromStr};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, ACCEPT},
    Method, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tauri::Window;

use crate::{error::HeaderError, http::multipart::FileKind, Result};

pub(crate) use self::delete::Delete;
pub(crate) use self::delete::{DeleteKeyKind, DeleteMethod, DeleteResponseController};
pub(crate) use self::list::{List, ListRequestMethod, ListResponseController};
pub(crate) use self::upload::{
    Upload, UploadContentType, UploadResponseController, UploadResponseErrorController,
    UploadResponseStatus, UploadResponseSuccuessController,
};

#[cfg(feature = "compress")]
use super::CompressedFormat;
use super::{
    AllowedImageFormat, BaseManager, DeleteError, DeleteResponse, Extra, ImageItem, Manage,
    UploadResult,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
/// 认证方式仅适用于 json 请求
pub enum AuthMethod {
    /// 通过请求头认证，key 为 None 时默认使用 Authorization
    Header {
        key: Option<String>,
        prefix: Option<String>,
    },
    Body {
        key: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Api {
    base_url: String,
    auth_method: AuthMethod,
    upload: Upload,
    list: List,
    delete: Delete,
}

impl Api {
    pub fn new<S: Into<String>>(
        base_url: S,
        auth_method: AuthMethod,
        upload: Upload,
        list: List,
        delete: Delete,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            auth_method,
            upload,
            list,
            delete,
        }
    }

    pub fn max_size(&self) -> u8 {
        self.upload.max_size
    }

    pub fn allowed_formats(&self) -> &[AllowedImageFormat] {
        &self.upload.allowed_formats
    }

    #[cfg(feature = "compress")]
    pub fn compressed_format(&self) -> &CompressedFormat {
        &self.upload.compressed_format
    }
}

#[derive(Debug)]
pub struct BaseApiManager {
    inner: BaseManager,
    token: String,
    api: Api,
}

impl BaseApiManager {
    pub(crate) fn new<S: Into<String>>(inner: BaseManager, token: S, api: &Api) -> Self {
        Self {
            inner,
            api: api.to_owned(),
            token: token.into(),
        }
    }

    pub fn allowed_formats(&self) -> Vec<AllowedImageFormat> {
        self.api.upload.allowed_formats.clone()
    }

    pub fn url(&self, path: &str) -> String {
        self.api.base_url.to_owned() + path
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());

        if let AuthMethod::Header { key, prefix } = &self.api.auth_method {
            let auth_key = key.as_deref().unwrap_or("Authorization");
            let key = HeaderName::from_str(auth_key).map_err(HeaderError::InvalidName)?;

            let token = match prefix {
                None => self.token.clone(),
                Some(p) => p.to_owned() + &self.token,
            };

            headers.insert(key, token.parse().unwrap());
        }

        Ok(headers)
    }

    pub async fn list(&self) -> Result<Vec<ImageItem>> {
        let response = match &self.api.list.method {
            ListRequestMethod::Get => {
                self.inner
                    .get(&self.url(&self.api.list.path), self.headers()?)
                    .await?
            }
            ListRequestMethod::Post { body } => {
                let mut body = body.clone();
                let headers = self.headers()?;

                if let AuthMethod::Body { key } = &self.api.auth_method {
                    body.insert(key.to_owned(), Value::String(self.token.clone()));
                }

                self.inner
                    .request(Method::POST, &self.api.list.path, headers)
                    .json(&body)
                    .send()
                    .await?
            }
        };

        self.api.list.controller.parse(response).await
    }

    async fn delete_by_delete(&self, kind: &DeleteKeyKind, id: &str) -> Result<Response> {
        // DELETE 删除认证方式只能是 headers
        let url = match kind {
            DeleteKeyKind::Path => self.url(&(self.api.delete.path.to_owned() + id)),
            DeleteKeyKind::Query { key } => format!(
                "{}{}?{}={}",
                self.api.base_url, self.api.delete.path, key, id
            ),
        };

        self.inner.delete(&url, self.headers()?).await
    }

    async fn delete_by_get(&self, kind: &DeleteKeyKind, id: &str) -> Result<Response> {
        // GET 删除认证方式只能是 headers
        let url = match kind {
            DeleteKeyKind::Path => self.url(&(self.api.delete.path.to_owned() + id)),
            DeleteKeyKind::Query { key } => format!(
                "{}{}?{}={}",
                self.api.base_url, self.api.delete.path, key, id
            ),
        };

        self.inner.get(&url, self.headers()?).await
    }

    async fn delete_by_post(
        &self,
        body: &Map<String, Value>,
        key: &str,
        id: &str,
    ) -> Result<Response> {
        let mut body = body.clone();

        body.insert(key.to_owned(), Value::String(id.to_owned()));

        if let AuthMethod::Body { key } = &self.api.auth_method {
            body.insert(key.clone(), Value::String(self.token.clone()));
        }

        self.inner
            .json(&self.url(&self.api.delete.path), self.headers()?, body)
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let resp = match &self.api.delete.method {
            DeleteMethod::Get { kind } => self.delete_by_get(kind, id).await?,
            DeleteMethod::Delete { kind } => self.delete_by_delete(kind, id).await?,
            DeleteMethod::Post { body, key } => self.delete_by_post(body, key, id).await?,
        };

        self.api.delete.controller.parse(resp).await
    }

    pub async fn upload(
        &self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
        form: Option<&[(&str, &str)]>,
    ) -> Result<UploadResult> {
        let headers = self.headers()?;

        debug!("超时时间：{}", self.api.upload.timeout);

        let response = match &self.api.upload.content_type {
            UploadContentType::Json { key } => {
                self.inner
                    .upload_json(
                        window,
                        id,
                        &self.url(&self.api.upload.path),
                        headers,
                        key,
                        image_path,
                        form,
                        self.api.upload.timeout,
                    )
                    .await?
            }
            UploadContentType::Multipart {
                file_part_name,
                file_kind,
            } => {
                self.inner
                    .upload_multipart(
                        window,
                        id,
                        &self.url(&self.api.upload.path),
                        headers,
                        image_path,
                        &file_part_name,
                        file_kind,
                        form,
                        self.api.upload.timeout,
                    )
                    .await?
            }
        };

        let image_item = self.api.upload.controller.parse(response).await?;

        info!("上传成功：{:?}", image_item);

        Ok(UploadResult::Response(image_item))
    }
}

#[async_trait]
impl Manage for BaseApiManager {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat> {
        self.api.upload.allowed_formats.clone()
    }

    fn support_stream(&self) -> bool {
        match &self.api.upload.content_type {
            UploadContentType::Json { .. } => true,
            UploadContentType::Multipart { file_kind, .. } => match file_kind {
                FileKind::Stream => true,
                _ => false,
            },
        }
    }

    async fn verify(&self) -> Result<Option<Extra>> {
        // TODO: api 类型的图床的 token 验证以后再实现
        Ok(None)
    }

    async fn get_all_images(&self) -> Result<Vec<ImageItem>> {
        self.list().await
    }

    async fn delete_image(&self, id: &str) -> Result<DeleteResponse> {
        match self.delete(id).await {
            Ok(()) => Ok(DeleteResponse {
                success: true,
                error: None,
            }),
            Err(e) => Ok(DeleteResponse {
                success: false,
                error: Some(DeleteError::Other(e.to_string())),
            }),
        }
    }

    async fn upload_image(
        &self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
    ) -> UploadResult {
        match self.upload(window, id, image_path, None).await {
            Ok(r) => r,
            Err(e) => {
                return UploadResult::Error {
                    code: e.as_string(),
                    detail: e,
                }
            }
        }
    }
}

trait SerdeValueParser {
    fn get_value_by_keys(&self, key_path: &str) -> Value;
}

impl SerdeValueParser for Value {
    fn get_value_by_keys(&self, key_path: &str) -> Value {
        let mut current_value = self;

        for key in key_path.split('.') {
            current_value = &current_value[key];
        }

        current_value.clone()
    }
}
