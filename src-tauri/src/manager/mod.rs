pub mod api;
pub mod chevereto;
pub mod git;
pub mod smms;

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use reqwest::{header::HeaderMap, Client, Method, RequestBuilder, Response};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, fmt::Display, path::Path, time::Duration};
use tauri::WebviewWindow;
use tokio::fs::{read, File};

use crate::{
    config::ManagerAuthConfigKind,
    error::{ConfigError, Up2bError},
    http::{
        json,
        multipart::{self, FileKind, UploadFile},
    },
    util::image::guess_mime_type_by_ext,
    Up2bResult,
};
#[cfg(feature = "compress")]
use {crate::config::CONFIG, crate::util::image::compress::compress};

use self::{
    api::BaseApiManager,
    chevereto::{Imgse, Imgtg},
    git::GitManager,
    smms::SmMs,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageItem {
    pub url: String,
    deleted_id: String,
    thumb: Option<String>,
}

#[derive(Debug)]
pub enum DeleteError {
    NotFound,
    Other(String),
}

impl Serialize for DeleteError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::NotFound => "图片不存在",
            Self::Other(s) => &s,
        };
        serializer.serialize_str(s)
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    success: bool,
    error: Option<DeleteError>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum UploadResult {
    Response(ImageItem),
    Error { detail: Up2bError, code: String },
}

pub type Extra = HashMap<String, String>;

#[async_trait]
pub trait Manage: Sync + Send {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat>;
    /// 上传时是否使用流，只有流式上传时前端才使用进度条
    fn support_stream(&self) -> bool;
    async fn verify(&self) -> Up2bResult<Option<Extra>>;
    async fn get_all_images(&self) -> Up2bResult<Vec<ImageItem>>;
    async fn delete_image(&self, id: &str) -> Up2bResult<DeleteResponse>;
    async fn upload_image(
        &self,
        window: Option<WebviewWindow>, // 命令行上传图片时不传入此参数
        id: u32,
        image_path: &Path,
    ) -> UploadResult;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum AllowedImageFormat {
    Jpeg,
    Png,
    Webp,
    Avif,
    Gif,
    Bmp,
}

#[cfg(feature = "compress")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CompressedFormat {
    JPEG,
    WEBP,
}

pub fn use_manager(
    using: &ManagerCode,
    auth_config: &ManagerAuthConfigKind,
) -> Up2bResult<Box<dyn Manage>> {
    let uploader: Box<dyn Manage> = match using {
        ManagerCode::Smms => match auth_config {
            ManagerAuthConfigKind::API { token, .. } => {
                let manager = SmMs::new(token.to_string());
                Box::new(manager)
            }
            _ => return Err(Up2bError::Config(ConfigError::Type(using.name()))),
        },
        ManagerCode::Imgse => match auth_config {
            ManagerAuthConfigKind::Chevereto {
                username,
                password,
                timeout,
                extra,
            } => {
                let imgse = Imgse::new(username, password, *timeout, extra.as_ref());
                Box::new(imgse)
            }
            _ => unreachable!(),
        },
        ManagerCode::Imgtg => match auth_config {
            ManagerAuthConfigKind::Chevereto {
                username,
                password,
                timeout,
                extra,
            } => {
                let imgtg = Imgtg::new(username, password, *timeout, extra.as_ref());
                Box::new(imgtg)
            }
            _ => unreachable!(),
        },
        ManagerCode::Github => match auth_config {
            ManagerAuthConfigKind::Git {
                token,
                username,
                repository,
                path,
                ..
            } => {
                let authorization = format!("Bearer {}", token);

                let github = GitManager::new(
                    "github",
                    "https://api.github.com",
                    HashMap::from([
                        (
                            "Accept".to_owned(),
                            "application/vnd.github+json".to_owned(),
                        ),
                        ("User-Agent".to_owned(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_owned()),
                        ("Authorization".to_owned(), authorization),
                        ("X-GitHub-Api-Version".to_owned(), "2022-11-28".to_owned()),
                    ]),
                    &token,
                    &username,
                    &repository,
                    path.as_deref(),
                    Some(180),
                    20,
                );
                Box::new(github)
            }
            _ => unreachable!(),
        },
        ManagerCode::Custom(s) => match auth_config {
            ManagerAuthConfigKind::API { token, api } => {
                let manager = BaseManager::new(
                    s.as_str(),
                    api.base_url(),
                    api.max_size(),
                    api.allowed_formats().to_vec(),
                    api.timeout().into(),
                    #[cfg(feature = "compress")]
                    api.compressed_format().clone(),
                );
                let custom = BaseApiManager::new(manager, token, api);

                Box::new(custom)
            }
            _ => unreachable!(),
        },
    };

    Ok(uploader)
}

async fn is_exceeded(
    image_bed_name: &str,
    image_path: &Path,
    max_size: u64,
    file_size: u64,
) -> Up2bResult<()> {
    if max_size >= file_size {
        return Ok(());
    }

    error!(
        "image size exceeds the maximum limit: {} > {}",
        file_size, max_size
    );
    return Err(Up2bError::OverSize(
        image_bed_name.to_string(),
        image_path.to_string_lossy().to_string(),
        max_size / 1024 / 1024,
        file_size / 1024 / 1024,
    ));
}

enum RequestWithBodyMethod {
    PUT,
    POST,
}

impl RequestWithBodyMethod {
    fn as_method(&self) -> Method {
        match self {
            RequestWithBodyMethod::PUT => Method::PUT,
            RequestWithBodyMethod::POST => Method::POST,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BaseManager {
    name: String,
    max_size: u8,
    base_url: String,
    client: Client,
    allowed_formats: Vec<AllowedImageFormat>,
    #[cfg(feature = "compress")]
    compressed_format: CompressedFormat,
    timeout: Duration,
}

impl BaseManager {
    fn new<S: Into<String>, A: Into<Vec<AllowedImageFormat>>>(
        name: S,
        base_url: S,
        max_size: u8,
        allowed_formats: A,
        timeout: Option<u8>,
        #[cfg(feature = "compress")] compressed_format: CompressedFormat,
    ) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            max_size,
            client: Client::new(),
            allowed_formats: allowed_formats.into(),
            timeout: Duration::from_secs(timeout.unwrap_or(5).into()),
            #[cfg(feature = "compress")]
            compressed_format,
        }
    }

    fn url(&self, path: &str) -> String {
        let char = path.chars().nth(0);

        if char != Some('/') {
            return format!("{}/{}", self.base_url, path);
        }

        self.base_url.to_owned() + path
    }

    async fn get(&self, url: &str, headers: HeaderMap) -> Up2bResult<Response> {
        trace!("发起 GET 请求：url={}, headers={:?}", url, headers);
        let resp = self.request(Method::GET, url, headers).send().await?;

        Ok(resp)
    }

    async fn delete(&self, url: &str, headers: HeaderMap) -> Up2bResult<Response> {
        let resp = self.request(Method::DELETE, url, headers).send().await?;

        Ok(resp)
    }

    // async fn post(&self, url: &str, headers: HeaderMap) -> Result<Response> {
    //     let resp = self.request(Method::POST, url, headers).send().await?;
    //
    //     Ok(resp)
    // }

    /// 不需要获取上传进度时使用此方法发起 json 请求
    async fn json<T: Serialize>(
        &self,
        method: RequestWithBodyMethod,
        url: &str,
        headers: HeaderMap,
        body: T,
    ) -> Up2bResult<Response> {
        let resp = self
            .request(method.as_method(), url, headers)
            .json(&body)
            .send()
            .await?;

        Ok(resp)
    }

    fn request(&self, method: Method, url: &str, headers: HeaderMap) -> RequestBuilder {
        self.client
            .request(method, url)
            .headers(headers)
            .timeout(self.timeout)
    }

    async fn compress(
        &self,
        #[cfg(feature = "compress")] window: Option<&Window>,
        file: File,
        image_path: &Path,
        #[cfg(feature = "compress")] filename: &str,
    ) -> Up2bResult<File> {
        let file_size = file.metadata().await?.len();

        let max_size = u64::from(self.max_size) * 1024 * 1024;

        #[cfg(not(feature = "compress"))]
        is_exceeded(&self.name, image_path, max_size, file_size).await?;

        #[cfg(feature = "compress")]
        let file = {
            let config = CONFIG.read().await.clone().unwrap();
            if !config.automatic_compression() {
                is_exceeded(&self.name, image_path, max_size, file_size).await?;
            }

            compress(
                window,
                max_size,
                file_size,
                filename,
                file,
                &self.compressed_format,
            )
            .await?
        };

        Ok(file)
    }

    async fn upload_json<T: Serialize>(
        &self,
        window: Option<WebviewWindow>,
        method: RequestWithBodyMethod,
        id: u32,
        url: &str,
        header: HeaderMap,
        key: &str,
        image_path: &Path,
        form: Option<T>,
    ) -> Up2bResult<Response> {
        // TODO: base64 上传对体积的限制待处理
        let file_data = read(image_path).await?;
        let file_data = general_purpose::STANDARD.encode(file_data);

        let mut body = serde_json::json!(form);

        debug!("序列化后除图片外的请求体：{}", body);

        match body {
            serde_json::Value::Null => {
                let mut map = Map::new();
                map.insert(key.to_owned(), Value::String(file_data));

                body = Value::Object(map);
            }
            serde_json::Value::Object(mut map) => {
                map.insert(key.to_owned(), Value::String(file_data));

                body = Value::Object(map);
            }
            _ => return Err(Up2bError::Other("错误的类型".to_owned())),
        };

        let builder = self.request(method.as_method(), url, header);

        let resp = json::upload(builder, window.as_ref(), body, id).await?;

        Ok(resp)
    }

    async fn upload_multipart(
        &self,
        window: Option<WebviewWindow>,
        id: u32,
        url: &str,
        header: HeaderMap,
        image_path: &Path,
        file_part_name: &str,
        file_kind: &FileKind,
        form: Option<&[(&str, &str)]>,
    ) -> Up2bResult<Response> {
        let file = File::open(&image_path).await?;
        let filename = image_path.file_name().unwrap().to_str().unwrap();

        let file = self
            .compress(
                #[cfg(feature = "compress")]
                window.as_ref(),
                file,
                image_path,
                #[cfg(feature = "compress")]
                filename,
            )
            .await?;

        let mime_type = guess_mime_type_by_ext(image_path.extension().unwrap().to_str().unwrap());

        debug!("guess mime type: {}", mime_type);

        let request_builder = self.request(Method::POST, url, header);

        let response = multipart::upload(
            request_builder,
            window.as_ref(),
            id,
            file_part_name,
            filename,
            UploadFile::new(file, file_kind),
            &mime_type,
            form,
        )
        .await
        .map_err(|e| {
            error!("上传图片出错：{}", e);
            e
        })?;

        Ok(response)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum ManagerKind {
    API,
    Git,
    Chevereto,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManagerCode {
    Smms, // 内置 smms 支持，与 Custom
    Imgse,
    Imgtg,
    Github,
    Custom(String),
}

impl Serialize for ManagerCode {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ManagerCode::Smms => serializer.serialize_str("SMMS"),
            ManagerCode::Imgse => serializer.serialize_str("IMGSE"),
            ManagerCode::Imgtg => serializer.serialize_str("IMGTG"),
            ManagerCode::Github => serializer.serialize_str("GITHUB"),
            ManagerCode::Custom(s) => {
                serializer.serialize_str(&format!("CUSTOM-{}", s.to_uppercase()))
            }
        }
    }
}

impl<'de> Deserialize<'de> for ManagerCode {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ManagerCodeVisitor;

        impl<'de> Visitor<'de> for ManagerCodeVisitor {
            type Value = ManagerCode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing ManagerCode")
            }

            fn visit_str<E>(self, value: &str) -> std::prelude::v1::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.starts_with("CUSTOM-") {
                    // If the value is "CUSTOM", parse it as Custom
                    let custom_value = value.trim_start_matches("CUSTOM-").to_string();
                    Ok(ManagerCode::Custom(custom_value))
                } else {
                    // Otherwise, convert the value to uppercase and try to match it with enum variants
                    match value.to_uppercase().as_str() {
                        "SMMS" => Ok(ManagerCode::Smms),
                        "IMGSE" => Ok(ManagerCode::Imgse),
                        "IMGTG" => Ok(ManagerCode::Imgtg),
                        "GITHUB" => Ok(ManagerCode::Github),
                        _ => Err(serde::de::Error::unknown_variant(
                            value,
                            &["SMMS", "IMGSE", "IMGTG", "GITHUB", "CUSTOM-{}"],
                        )),
                    }
                }
            }
        }

        deserializer.deserialize_str(ManagerCodeVisitor)
    }
}

impl Display for ManagerCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for ManagerCode {
    fn default() -> Self {
        ManagerCode::Smms
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManagerItem {
    pub key: ManagerCode,
    pub name: String,
    pub index: Option<&'static str>,
    pub r#type: ManagerKind,
}

impl ManagerCode {
    pub fn name(&self) -> String {
        match self {
            ManagerCode::Smms => "sm.ms".to_owned(),
            ManagerCode::Imgse => "imgse.com".to_owned(),
            ManagerCode::Imgtg => "imgtg.com".to_owned(),
            ManagerCode::Github => "github.com".to_owned(),
            ManagerCode::Custom(s) => "CUSTOM-".to_owned() + s,
        }
    }

    pub fn index(&self) -> Option<&'static str> {
        match self {
            ManagerCode::Smms => Some("https://sm.ms"),
            ManagerCode::Imgse => Some("https://imgse.com"),
            ManagerCode::Imgtg => Some("https://imgtg.com"),
            ManagerCode::Github => Some("https://github.com"),
            _ => None,
        }
    }

    pub fn to_manager_item(self) -> ManagerItem {
        match self {
            ManagerCode::Smms => ManagerItem {
                name: self.name(),
                index: self.index(),
                key: self,
                r#type: ManagerKind::API,
            },
            ManagerCode::Imgse => ManagerItem {
                name: self.name(),
                index: self.index(),
                key: self,
                r#type: ManagerKind::Chevereto,
            },
            ManagerCode::Imgtg => ManagerItem {
                name: self.name(),
                index: self.index(),
                key: self,
                r#type: ManagerKind::Chevereto,
            },
            ManagerCode::Github => ManagerItem {
                name: self.name(),
                index: self.index(),
                key: self,
                r#type: ManagerKind::Git,
            },
            ManagerCode::Custom(ref s) => ManagerItem {
                name: s.clone(),
                index: None,
                key: self,
                r#type: ManagerKind::API,
            },
        }
    }
}

lazy_static! {
    pub(crate) static ref MANAGERS: [ManagerItem; 4] = [
        ManagerCode::Smms.to_manager_item(),
        ManagerCode::Imgse.to_manager_item(),
        ManagerCode::Imgtg.to_manager_item(),
        ManagerCode::Github.to_manager_item()
    ];
}
