use std::{collections::HashMap, path::Path, str::FromStr as _};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION},
    Method, StatusCode,
};
use serde_json::Value;
use tauri::Window;

#[cfg(feature = "compress")]
use super::CompressedFormat;
use super::{
    AllowedImageFormat, BaseManager, DeleteResponse, Extra, ImageItem, Manage,
    RequestWithBodyMethod, UploadResult,
};

use crate::{
    error::{GitError, HeaderError, PathError, UploadError},
    manager::DeleteError,
    util::time::now,
    Error, Result,
};

#[derive(Debug)]
pub(super) struct GitManager {
    inner: BaseManager,
    // auth_method: AuthMethod, // 当前只支持 github，认证方式暂时只用 headers
    headers: Option<HashMap<String, String>>,
    token: String,
}

impl GitManager {
    pub(super) fn new<H: Into<Option<HashMap<String, String>>>>(
        name: &str,
        base_url: &str,
        // auth_method: AuthMethod,
        headers: H,
        token: &str,
        username: &str,
        repository: &str,
        path: Option<&str>,
        timeout: Option<u8>,
        max_size: u8,
    ) -> Self {
        let inner = BaseManager::new(
            name,
            &format!(
                "{}/repos/{}/{}/contents/{}",
                base_url,
                username,
                repository,
                path.unwrap_or("up2b")
            ),
            max_size,
            vec![
                AllowedImageFormat::Jpeg,
                AllowedImageFormat::Png,
                AllowedImageFormat::Gif,
                AllowedImageFormat::Bmp,
                AllowedImageFormat::Webp,
                AllowedImageFormat::Avif,
            ],
            timeout,
            #[cfg(feature = "compress")]
            CompressedFormat::WEBP,
        );

        Self {
            inner,
            headers: headers.into(),
            token: token.into(),
        }
    }

    pub fn allowed_formats(&self) -> &[AllowedImageFormat] {
        &self.inner.allowed_formats
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        match &self.headers {
            None => {
                headers.insert(ACCEPT, "application/json".parse().unwrap());
            }
            Some(map) => {
                for (k, v) in map.iter() {
                    let key = HeaderName::from_str(&k).map_err(HeaderError::InvalidName)?;
                    let val = HeaderValue::from_str(&v).map_err(HeaderError::InvalidValue)?;
                    headers.insert(key, val);
                }
            }
        }

        headers.insert(
            AUTHORIZATION,
            ("Bearer ".to_owned() + &self.token).parse().unwrap(),
        );

        debug!("请求头: {:?}", headers);
        // if let AuthMethod::Header { key, prefix } = &self.auth_method {
        //     let auth_key = key.as_deref().unwrap_or("Authorization");
        //     let key = HeaderName::from_str(auth_key).map_err(HeaderError::InvalidName)?;
        //
        //     let token = match prefix {
        //         None => self.token.clone(),
        //         Some(p) => p.to_owned() + &self.token,
        //     };
        //
        //     headers.insert(key, token.parse().unwrap());
        // }

        Ok(headers)
    }

    fn parse_images(&self, items: &[Value]) -> Result<Vec<ImageItem>> {
        let mut image_items = Vec::with_capacity(items.len());

        for item in items.iter() {
            let download_url = item.get_string("download_url")?;
            let sha = item.get_string("sha")?;
            let url = item.get_string("url")?;
            image_items.push(ImageItem {
                url: download_url,
                deleted_id: format!("{}---{}", url, sha),
                thumb: None,
            })
        }

        Ok(image_items)
    }

    pub async fn list(&self) -> Result<Vec<ImageItem>> {
        let resp = self
            .inner
            .get(&self.inner.base_url, self.headers()?)
            .await?;

        let status = resp.status();
        let json: Value = resp.json().await?;

        if status != StatusCode::OK {
            let message = json.get_string("message")?;
            error!("获取图片列表错误，状态码：{}，错误：{}", status, message);

            let error: GitError = message.into();

            return Err(error.into());
        }

        match json {
            Value::Array(items) => self.parse_images(&items),
            _ => unreachable!(),
        }
    }

    const DELETE_MESSAGE: &'static str = "up2b: delete the picture that is no longer used";

    pub async fn delete(&self, s: &str) -> Result<DeleteResponse> {
        // s 为 url 和 sha 合并后的字符串，用"---"分隔
        let v: Vec<&str> = s.splitn(2, "---").collect();
        let url = v[0];
        let sha = v[1];

        let data = HashMap::from([("sha", sha), ("message", Self::DELETE_MESSAGE)]);

        let resp = self
            .inner
            .request(Method::DELETE, url, self.headers()?)
            .json(&data)
            .send()
            .await?;

        let status = resp.status();
        if status != StatusCode::OK {
            let json: Value = resp.json().await?;
            let message = json.get_string("message")?;
            error!("删除失败，状态码：{}，错误：{}", status, message);

            // TODO: 以后处理图片不存在的错误
            return Ok(DeleteResponse {
                success: false,
                error: Some(DeleteError::Other(message)),
            });
        }

        Ok(DeleteResponse {
            success: true,
            error: None,
        })
    }

    async fn upload(
        &self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
    ) -> Result<UploadResult> {
        let filename = match image_path.file_name() {
            Some(n) => n.to_os_string().into_string().unwrap(),
            None => return Err(PathError::NotFile.into()),
        };

        let now = now()?;

        let form = HashMap::from([("message".to_owned(), "up2b: ".to_owned() + &filename)]);

        let parts: Vec<&str> = filename.rsplitn(2, '.').collect();

        let filename_with_timestamp = format!("{}_{}.{}", parts[1], now.as_millis(), parts[0]);

        let url = &self.inner.url(&filename_with_timestamp);
        let resp = self
            .inner
            .upload_json(
                window,
                RequestWithBodyMethod::PUT,
                id,
                url,
                self.headers()?,
                "content",
                image_path,
                Some(form),
            )
            .await?;

        let status = resp.status();
        let json: Value = resp.json().await?;
        if status != StatusCode::CREATED {
            let message = json.get_string("message")?;
            error!("上传失败，状态码：{}，错误：{}", status, message);

            return Err(UploadError::Error(message).into());
        }

        let content = &json["content"];

        let download_url = content.get_string("download_url")?;
        let sha = content.get_string("sha")?;
        let url = content.get_string("url")?;

        info!("图片已上传：path={:?}, url={}", image_path, download_url);

        Ok(UploadResult::Response(ImageItem {
            url: download_url,
            deleted_id: format!("{}---{}", url, sha),
            thumb: None,
        }))
    }
}

trait ValueGetter {
    fn get_string(&self, key: &str) -> Result<String>;
    // fn get_array(&self, key: &str) -> Result<Vec<Value>>;
}

impl ValueGetter for Value {
    fn get_string(&self, key: &str) -> Result<String> {
        match &self[key] {
            Value::String(s) => Ok(s.to_owned()),
            Value::Null => return Err(Error::KeyNotFound(key.to_owned())),
            _ => return Err(Error::KeyNotMatch(key.to_owned())),
        }
    }

    // fn get_array(&self, key: &str) -> Result<Vec<Value>> {
    //     match self[key] {
    //         Value::Array(s) => Ok(s),
    //         Value::Null => return Err(Error::KeyNotFound(key.to_owned())),
    //         _ => return Err(Error::KeyNotMatch(key.to_owned())),
    //     }
    // }
}

#[async_trait]
impl<'r> Manage for GitManager {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat> {
        self.allowed_formats().to_owned()
    }

    fn support_stream(&self) -> bool {
        true
    }

    async fn verify(&self) -> Result<Option<Extra>> {
        // TODO: 验证以后再实现
        Ok(None)
    }

    async fn get_all_images(&self) -> Result<Vec<ImageItem>> {
        self.list().await
    }

    async fn delete_image(&self, id: &str) -> Result<DeleteResponse> {
        match self.delete(id).await {
            Ok(r) => Ok(r),
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
        match self.upload(window, id, image_path).await {
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
