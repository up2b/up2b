use std::{path::Path, str::FromStr};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, ACCEPT},
    Method, Response, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tauri::Window;

use crate::{error::HeaderError, http::multipart::FileKind, Error, Result};

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
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum UploadContentType {
    Json {
        key: String,
    },
    Multipart {
        file_part_name: String,
        file_kind: FileKind,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadResponseController {
    image_url_key: String,
    /// 有的图床不提供缩略图
    thumb_key: Option<String>,
    deleted_id_key: String,
}

impl UploadResponseController {
    pub fn new<S: Into<String>, O: Into<Option<String>>>(
        image_url_key: S,
        thumb_key: O,
        deleted_id_key: S,
    ) -> Self {
        Self {
            image_url_key: image_url_key.into(),
            thumb_key: thumb_key.into(),
            deleted_id_key: deleted_id_key.into(),
        }
    }

    pub async fn parse(&self, response: Response) -> Result<ImageItem> {
        let json: Value = response.json().await?;

        debug!("响应体：{}", json);

        let url = match json.get_value_by_keys(&self.image_url_key) {
            Value::String(s) => s,
            Value::Null => return Err(Error::Other("没有图片链接".to_owned())),
            _ => return Err(Error::Other("类型错误".to_owned())),
        };
        let deleted_id = match json.get_value_by_keys(&self.deleted_id_key) {
            Value::String(s) => s,
            Value::Null => return Err(Error::Other("没有删除 id".to_owned())),
            _ => return Err(Error::Other("类型错误".to_owned())),
        };

        match &self.thumb_key {
            None => Ok(ImageItem {
                url,
                deleted_id,
                thumb: None,
            }),
            Some(k) => {
                let thumb = match json.get_value_by_keys(k) {
                    Value::String(s) => Some(s),
                    Value::Null => None,
                    _ => return Err(Error::Other("类型错误".to_owned())),
                };

                Ok(ImageItem {
                    url,
                    deleted_id,
                    thumb,
                })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upload {
    path: String,
    max_size: u8,
    allowed_formats: Vec<AllowedImageFormat>,
    #[cfg(feature = "compress")]
    compressed_format: CompressedFormat,
    content_type: UploadContentType,
    /// 请求体中图片之外的其他部分
    other_body: Option<Map<String, Value>>,
    controller: UploadResponseController,
    timeout: u64,
}

impl Upload {
    pub fn new<T: Into<Option<u64>>, M: Into<Option<Map<String, Value>>>>(
        url: &str,
        max_size: u8,
        allowed_formats: Vec<AllowedImageFormat>,
        #[cfg(feature = "compress")] compressed_format: CompressedFormat,
        content_type: UploadContentType,
        other_body: M,
        controller: UploadResponseController,
        timeout: T,
    ) -> Self {
        let secs: Option<u64> = timeout.into();
        let timeout = {
            let secs = secs.unwrap_or(5);

            if secs == 0 {
                5
            } else {
                secs
            }
        };
        Self {
            path: url.into(),
            max_size,
            allowed_formats,
            #[cfg(feature = "compress")]
            compressed_format,
            content_type,
            controller,
            other_body: other_body.into(),
            timeout,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum ListRequestMethod {
    Get,
    Post { body: Map<String, Value> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List {
    path: String,
    controller: ListResponseController,
    method: ListRequestMethod,
}

impl List {
    pub fn new(url: &str, controller: ListResponseController, method: ListRequestMethod) -> Self {
        Self {
            path: url.to_owned(),
            controller,
            method,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListResponseController {
    items_key: String,
    image_url_key: String,
    thumb_key: Option<String>,
    deleted_id_key: String,
}

impl ListResponseController {
    pub fn new<S: Into<String>, O: Into<Option<String>>>(
        items_key: S,
        image_url_key: S,
        deleted_id_key: S,
        thumb_key: O,
    ) -> Self {
        Self {
            items_key: items_key.into(),
            image_url_key: image_url_key.into(),
            deleted_id_key: deleted_id_key.into(),
            thumb_key: thumb_key.into(),
        }
    }

    async fn parse(&self, response: Response) -> Result<Vec<ImageItem>> {
        let json: Value = response.json().await?;

        let items = match json.get_value_by_keys(&self.items_key) {
            Value::Array(arr) => arr,
            _ => return Err(Error::Other("通过 items_key 无法获取列表".to_owned())),
        };

        println!("{:?}", items);

        let mut images = Vec::with_capacity(items.len());

        for item in items.iter() {
            let url = match item.get_value_by_keys(&self.image_url_key) {
                Value::String(s) => s,
                _ => return Err(Error::Other("没有图片链接".to_owned())),
            };
            let deleted_id = match item.get_value_by_keys(&self.deleted_id_key) {
                Value::String(s) => s,
                _ => return Err(Error::Other("没有删除 id".to_owned())),
            };

            let image_item = match &self.thumb_key {
                None => ImageItem {
                    url,
                    deleted_id,
                    thumb: None,
                },
                Some(k) => {
                    let thumb = match item.get_value_by_keys(k) {
                        Value::String(s) => Some(s),
                        _ => None,
                    };

                    debug!("thumb: key={} value={:?}", k, thumb);

                    ImageItem {
                        url,
                        deleted_id,
                        thumb,
                    }
                }
            };

            images.push(image_item);
        }

        debug!("图片列表：{:?}", images);

        Ok(images)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DeleteKeyKind {
    Path,
    Query { key: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DeleteMethod {
    Get {
        kind: DeleteKeyKind,
    },
    Delete {
        kind: DeleteKeyKind,
    },
    Post {
        body: Map<String, Value>,
        key: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DeleteResponseController {
    /// status-code = 200 才成功，错误时返回 unkown
    Status,
    Json {
        /// 成功与否的 key
        key: String,
        /// 成功的值应是多少
        should_be: Value,
        /// 失败时的消息 key，为 None 时如果出错则返回 unkown
        message_key: Option<String>,
    },
}

impl DeleteResponseController {
    async fn parse(&self, response: Response) -> Result<()> {
        match self {
            DeleteResponseController::Status => {
                if response.status() != StatusCode::OK {
                    return Err(Error::Other("unkown".to_owned()));
                }
            }
            DeleteResponseController::Json {
                key,
                should_be,
                message_key,
            } => {
                let json: Value = response.json().await?;

                let value = json.get_value_by_keys(&key);

                if value.is_null() {
                    return Err(Error::Other("无法获取删除状态值".to_owned()));
                }

                debug!("删除状态：{}", value);

                if &value != should_be {
                    match message_key {
                        None => return Err(Error::Other("unkown".to_owned())),
                        Some(k) => match json.get(k) {
                            None => return Err(Error::Other("unkown".to_owned())),
                            Some(msg) => {
                                return Err(Error::Other(msg.as_str().unwrap().to_owned()))
                            }
                        },
                    }
                }
            }
        };

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Delete {
    path: String,
    method: DeleteMethod,
    controller: DeleteResponseController,
}

impl Delete {
    pub fn new(url: &str, method: DeleteMethod, controller: DeleteResponseController) -> Self {
        Self {
            path: url.to_owned(),
            method,
            controller,
        }
    }
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
