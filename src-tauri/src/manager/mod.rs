pub mod api;
pub mod chevereto;
pub mod smms;

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use reqwest::{header::HeaderMap, Client, Method, RequestBuilder, Response};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, fmt::Display, path::Path};
use tauri::Window;
use tokio::fs::{read, File};

use crate::{
    config::ManagerAuthConfigKind,
    error::{ConfigError, Error},
    http::{
        json,
        multipart::{upload, FileKind, UploadFile},
    },
    util::image::guess_mime_type_by_ext,
    Result,
};
#[cfg(feature = "compress")]
use {crate::config::CONFIG, crate::util::image::compress::compress};

use self::{
    api::BaseApiManager,
    chevereto::{Imgse, Imgtg},
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
    Error { detail: Error, code: String },
}

pub type Extra = HashMap<String, String>;

#[async_trait]
pub trait Manage: Sync + Send {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat>;
    /// 上传时是否使用流，只有流式上传时前端才使用进度条
    fn support_stream(&self) -> bool;
    async fn verify(&self) -> Result<Option<Extra>>;
    async fn get_all_images(&self) -> Result<Vec<ImageItem>>;
    async fn delete_image(&self, id: &str) -> Result<DeleteResponse>;
    async fn upload_image(
        &self,
        window: Option<Window>, // 命令行上传图片时不传入此参数
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
    // Avif, // NOTE: 多数图床不支持此格式,暂不启用
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
) -> Result<Box<dyn Manage>> {
    let uploader: Box<dyn Manage> = match using {
        ManagerCode::Smms => match auth_config {
            ManagerAuthConfigKind::API { token, .. } => {
                let manager = SmMs::new(token.to_string());
                Box::new(manager)
            }
            _ => return Err(Error::Config(ConfigError::Type(using.name()))),
        },
        ManagerCode::Imgse => match auth_config {
            ManagerAuthConfigKind::Chevereto {
                username,
                password,
                extra,
            } => {
                let imgse = Imgse::new(username, password, extra.as_ref());
                Box::new(imgse)
            }
            _ => unreachable!(),
        },
        ManagerCode::Imgtg => match auth_config {
            ManagerAuthConfigKind::Chevereto {
                username,
                password,
                extra,
            } => {
                let imgtg = Imgtg::new(username, password, extra.as_ref());
                Box::new(imgtg)
            }
            _ => unreachable!(),
        },
        ManagerCode::Custom(s) => match auth_config {
            ManagerAuthConfigKind::API { token, api } => {
                let manager = BaseManager::new(
                    s,
                    api.max_size(),
                    api.allowed_formats().to_vec(),
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
) -> Result<()> {
    if max_size >= file_size {
        return Ok(());
    }

    error!(
        "image size exceeds the maximum limit: {} > {}",
        file_size, max_size
    );
    return Err(Error::OverSize(
        image_bed_name.to_string(),
        image_path.to_string_lossy().to_string(),
        max_size / 1024 / 1024,
        file_size / 1024 / 1024,
    ));
}

#[derive(Debug, Clone)]
pub(crate) struct BaseManager {
    name: String,
    max_size: u8,
    client: Client,
    allowed_formats: Vec<AllowedImageFormat>,
    #[cfg(feature = "compress")]
    compressed_format: CompressedFormat,
}

impl BaseManager {
    fn new<S: Into<String>>(
        name: S,
        max_size: u8,
        allowed_formats: Vec<AllowedImageFormat>,
        #[cfg(feature = "compress")] compressed_format: CompressedFormat,
    ) -> Self {
        Self {
            name: name.into(),
            max_size,
            client: Client::new(),
            allowed_formats,
            #[cfg(feature = "compress")]
            compressed_format,
        }
    }

    async fn get(&self, url: &str, headers: HeaderMap) -> Result<Response> {
        let resp = self.request(Method::GET, url, headers).send().await?;

        Ok(resp)
    }

    async fn delete(&self, url: &str, headers: HeaderMap) -> Result<Response> {
        let resp = self.request(Method::DELETE, url, headers).send().await?;

        Ok(resp)
    }

    async fn post(&self, url: &str, headers: HeaderMap) -> Result<Response> {
        let resp = self.request(Method::POST, url, headers).send().await?;

        Ok(resp)
    }

    async fn json<T: Serialize>(&self, url: &str, headers: HeaderMap, body: T) -> Result<Response> {
        let resp = self
            .client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        Ok(resp)
    }

    fn request(&self, method: Method, url: &str, headers: HeaderMap) -> RequestBuilder {
        self.client.request(method, url).headers(headers)
    }

    async fn compress(
        &self,
        #[cfg(feature = "compress")] window: Option<&Window>,
        file: File,
        image_path: &Path,
        #[cfg(feature = "compress")] filename: &str,
    ) -> Result<File> {
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

    async fn upload_json(
        &self,
        window: Option<Window>,
        id: u32,
        url: &str,
        header: HeaderMap,
        key: &str,
        image_path: &Path,
        form: Option<&[(&str, &str)]>,
        timeout: u64,
    ) -> Result<Response> {
        // TODO: base64 上传对体积的限制待处理
        let file_data = read(image_path).await?;
        let file_data = general_purpose::STANDARD.encode(file_data);

        let mut body = serde_json::json!(form);

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
            _ => return Err(Error::Other("错误的类型".to_owned())),
        };

        let builder = self.request(Method::POST, url, header);

        let resp = json::upload(builder, window.as_ref(), body, id, timeout).await?;

        Ok(resp)
    }

    async fn upload_multipart(
        &self,
        window: Option<Window>,
        id: u32,
        url: &str,
        header: HeaderMap,
        image_path: &Path,
        file_part_name: &str,
        file_kind: &FileKind,
        form: Option<&[(&str, &str)]>,
        timeout: u64,
    ) -> Result<Response> {
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

        let response = upload(
            &self.client,
            window.as_ref(),
            url,
            header,
            id,
            file_part_name,
            filename,
            UploadFile::new(file, file_kind),
            &mime_type,
            form,
            timeout,
        )
        .await
        .map_err(|e| {
            error!("上传图片出错：{}", e);
            e
        })?;

        Ok(response)
    }

    async fn upload(
        &self,
        window: Option<Window>,
        id: u32,
        url: &str,
        header: HeaderMap,
        image_path: &Path,
        file_kind: &FileKind,
        file_part_name: &str,
        form: Option<&[(&str, &str)]>,
    ) -> Result<Response> {
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

        let response = upload(
            &self.client,
            window.as_ref(),
            url,
            header,
            id,
            file_part_name,
            filename,
            UploadFile::new(file, file_kind),
            &mime_type,
            form,
            60,
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
    Chevereto,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManagerCode {
    Smms, // 内置 smms 支持，与 Custom
    Imgse,
    Imgtg,
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
                        _ => Err(serde::de::Error::unknown_variant(
                            value,
                            &["SMMS", "IMGSE", "IMGTG", "CUSTOM"],
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
            ManagerCode::Custom(s) => "CUSTOM-".to_owned() + s,
        }
    }

    pub fn index(&self) -> Option<&'static str> {
        match self {
            ManagerCode::Smms => Some("https://sm.ms"),
            ManagerCode::Imgse => Some("https://imgse.com"),
            ManagerCode::Imgtg => Some("https://imgtg.com"),
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
    pub(crate) static ref MANAGERS: [ManagerItem; 3] = [
        ManagerCode::Smms.to_manager_item(),
        ManagerCode::Imgse.to_manager_item(),
        ManagerCode::Imgtg.to_manager_item(),
    ];
}
