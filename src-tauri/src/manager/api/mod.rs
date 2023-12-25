use std::{path::Path, str::FromStr};

use reqwest::{
    header::{HeaderMap, HeaderName, ACCEPT},
    Method, Response, StatusCode,
};
use serde_json::{Map, Value};
use tauri::Window;

use crate::{error::HeaderError, http::multipart::FileKind, Error, Result};

use super::{AllowedImageFormat, BaseManager, CompressedFormat, ImageItem, UploadResult};

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

pub enum UploadContentType {
    Json {
        key: String,
    },
    Multipart {
        file_part_name: String,
        file_kind: FileKind,
    },
}

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

        let url = match json.get(&self.image_url_key) {
            None => return Err(crate::Error::Other("没有图片链接".to_owned())),
            Some(v) => v.as_str().unwrap().to_owned(),
        };
        let deleted_id = match json.get(&self.deleted_id_key) {
            None => return Err(crate::Error::Other("没有删除 id".to_owned())),
            Some(v) => v.as_str().unwrap().to_owned(),
        };

        match &self.thumb_key {
            None => Ok(ImageItem {
                url,
                deleted_id,
                thumb: None,
            }),
            Some(k) => {
                let thumb = match json.get(k) {
                    None => None,
                    Some(v) => Some(v.as_str().unwrap().to_owned()),
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

pub struct Upload {
    url: String,
    max_size: u64,
    allowed_formats: Vec<AllowedImageFormat>,
    compressed_format: CompressedFormat,
    content_type: UploadContentType,
    controller: UploadResponseController,
}

impl Upload {
    pub fn new(
        url: &str,
        max_size: u64,
        allowed_formats: Vec<AllowedImageFormat>,
        compressed_format: CompressedFormat,
        content_type: UploadContentType,
        controller: UploadResponseController,
    ) -> Self {
        Self {
            url: url.into(),
            max_size,
            allowed_formats,
            compressed_format,
            content_type,
            controller,
        }
    }
}

enum ListRequestMethod {
    Get,
    Post { body: Map<String, Value> },
}

struct List {
    url: String,
    controller: ListResponseController,
    method: ListRequestMethod,
}

pub struct ListResponseController {
    items_key: String,
    image_url_key: String,
    thumb_key: Option<String>,
    deleted_id_key: String,
}

impl ListResponseController {
    async fn parse(&self, response: Response) -> Result<Vec<ImageItem>> {
        let json: Value = response.json().await?;

        let items = match json.get(&self.items_key) {
            None => return Err(Error::Other("通过 items_key 无法获取列表".to_owned())),
            Some(v) => v.as_array().unwrap(),
        };

        let mut images = Vec::with_capacity(items.len());

        for item in items.iter() {
            let url = match item.get(&self.image_url_key) {
                None => return Err(Error::Other("没有图片链接".to_owned())),
                Some(v) => v.as_str().unwrap().to_owned(),
            };
            let deleted_id = match item.get(&self.deleted_id_key) {
                None => return Err(Error::Other("没有删除 id".to_owned())),
                Some(v) => v.as_str().unwrap().to_owned(),
            };

            let image_item = match &self.thumb_key {
                None => ImageItem {
                    url,
                    deleted_id,
                    thumb: None,
                },
                Some(k) => {
                    let thumb = match json.get(k) {
                        None => None,
                        Some(v) => Some(v.as_str().unwrap().to_owned()),
                    };

                    ImageItem {
                        url,
                        deleted_id,
                        thumb,
                    }
                }
            };

            images.push(image_item);
        }

        Ok(images)
    }
}

enum DeleteGetKind {
    Path,
    Query { key: String },
}

enum DeleteMethod {
    Get {
        kind: DeleteGetKind,
    },
    Post {
        body: Map<String, Value>,
        key: String,
    },
}

enum DeleteResponseController {
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

                match json.get(&key) {
                    None => return Err(Error::Other("无法获取删除状态值".to_owned())),
                    Some(v) => {
                        if v != should_be {
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
            }
        };

        Ok(())
    }
}

struct Delete {
    url: String,
    method: DeleteMethod,
    controller: DeleteResponseController,
}

pub struct Api {
    upload: Upload,
    list: List,
    delete: Delete,
}

impl Api {
    pub fn new(upload: Upload, list: List, delete: Delete) -> Self {
        Self {
            upload,
            list,
            delete,
        }
    }
}

struct BaseApiManager {
    inner: BaseManager,
    auth_method: AuthMethod,
    token: String,
    api: Api,
}

impl BaseApiManager {
    fn new<S: Into<String>>(
        inner: BaseManager,
        auth_method: AuthMethod,
        token: S,
        api: Api,
    ) -> Self {
        Self {
            inner,
            auth_method,
            api,
            token: token.into(),
        }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());

        if let AuthMethod::Header { key, prefix } = &self.auth_method {
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

    async fn list(&self) -> Result<Vec<ImageItem>> {
        let response = match &self.api.list.method {
            ListRequestMethod::Get => self.inner.get(&self.api.list.url, self.headers()?).await?,
            ListRequestMethod::Post { body } => {
                let mut body = body.clone();
                let headers = self.headers()?;

                if let AuthMethod::Body { key } = &self.auth_method {
                    body.insert(key.to_owned(), Value::String(self.token.clone()));
                }

                self.inner
                    .request(Method::POST, &self.api.list.url, headers)
                    .json(&body)
                    .send()
                    .await?
            }
        };

        self.api.list.controller.parse(response).await
    }

    async fn delete_by_get(&self, kind: &DeleteGetKind, id: &str) -> Result<Response> {
        // GET 删除认证方式只能是 headers
        let url = match kind {
            DeleteGetKind::Path => self.api.delete.url.to_owned() + id,
            DeleteGetKind::Query { key } => format!("{}?{}={}", self.api.delete.url, key, id),
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

        if let AuthMethod::Body { key } = &self.auth_method {
            body.insert(key.clone(), Value::String(self.token.clone()));
        }

        self.inner
            .json(&self.api.delete.url, self.headers()?, body)
            .await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let resp = match &self.api.delete.method {
            DeleteMethod::Get { kind } => self.delete_by_get(kind, id).await?,
            DeleteMethod::Post { body, key } => self.delete_by_post(body, key, id).await?,
        };

        self.api.delete.controller.parse(resp).await
    }

    async fn upload(
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
                        &self.api.upload.url,
                        headers,
                        key,
                        image_path,
                        form,
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
                        &self.api.upload.url,
                        headers,
                        image_path,
                        &file_part_name,
                        file_kind,
                        form,
                    )
                    .await?
            }
        };

        let image_item = self.api.upload.controller.parse(response).await?;

        Ok(UploadResult::Response(image_item))
    }
}
