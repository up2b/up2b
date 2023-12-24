use std::{path::Path, str::FromStr};

use reqwest::{
    header::{HeaderMap, HeaderName, ACCEPT},
    Method, Response,
};
use serde_json::Value;
use tauri::Window;

use crate::{error::HeaderError, http::multipart::FileKind, Error, Result};

use super::{AllowedImageFormat, BaseManager, CompressedFormat, ImageItem, UploadResult};

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
    deleted_id: String,
}

impl UploadResponseController {
    pub fn new<S: Into<String>, O: Into<Option<String>>>(
        image_url_key: S,
        thumb_key: O,
        deleted_id: S,
    ) -> Self {
        Self {
            image_url_key: image_url_key.into(),
            thumb_key: thumb_key.into(),
            deleted_id: deleted_id.into(),
        }
    }

    pub async fn parse(&self, response: Response) -> Result<ImageItem> {
        let json: Value = response.json().await?;

        let url = match json.get(&self.image_url_key) {
            None => return Err(crate::Error::Other("没有图片链接".to_owned())),
            Some(v) => v.as_str().unwrap().to_owned(),
        };
        let deleted_id = match json.get(&self.deleted_id) {
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

pub struct Api {
    upload: Upload,
}

impl Api {
    pub fn new(upload: Upload) -> Self {
        Self { upload }
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
