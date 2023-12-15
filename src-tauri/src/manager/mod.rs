pub mod chevereto;
pub mod smms;

use async_trait::async_trait;
use reqwest::{header::HeaderMap, Client, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::Path};
use tauri::Window;
use tokio::fs::File;

use crate::{
    config::ManagerAuthConfigKind,
    error::{ConfigError, Error},
    http::multipart::{upload, FileKind, UploadFile},
    util::image::guess_mime_type_by_ext,
    Result,
};
#[cfg(feature = "compress")]
use {crate::config::CONFIG, crate::util::image::compress::compress};

use self::{
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

#[derive(Debug, Serialize, Clone)]
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
#[derive(Debug, Clone)]
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
            ManagerAuthConfigKind::API { token } => {
                let manager = SmMs::new(token.to_string());
                Box::new(manager)
            }
            _ => return Err(Error::Config(ConfigError::Type("sm.ms".to_string()))),
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
            _ => return Err(Error::Config(ConfigError::Type("imgse.com".to_string()))),
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
            _ => return Err(Error::Config(ConfigError::Type("imgtg.com".to_string()))),
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
struct BaseManager {
    name: String,
    base_url: String,
    max_size: u64,
    client: Client,
    file_part_name: String,
    file_kind: FileKind,
    allowed_formats: Vec<AllowedImageFormat>,
    #[cfg(feature = "compress")]
    compressed_format: CompressedFormat,
}

impl BaseManager {
    fn new<S: Into<String>>(
        name: S,
        base_url: S,
        max_size: u64,
        file_part_name: S,
        file_kind: FileKind,
        allowed_formats: Vec<AllowedImageFormat>,
        #[cfg(feature = "compress")] compressed_format: CompressedFormat,
    ) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            max_size,
            file_part_name: file_part_name.into(),
            file_kind,
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

    async fn post(&self, url: &str, headers: HeaderMap) -> Result<Response> {
        let resp = self.request(Method::POST, url, headers).send().await?;

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

        #[cfg(not(feature = "compress"))]
        is_exceeded(&self.name, image_path, self.max_size, file_size).await?;

        #[cfg(feature = "compress")]
        let file = {
            let config = CONFIG.read().await.clone().unwrap();
            if !config.automatic_compression() {
                is_exceeded(&self.name, image_path, self.max_size, file_size).await?;
            }

            compress(
                window,
                self.max_size,
                file_size,
                filename,
                file,
                &self.compressed_format,
            )
            .await?
        };

        Ok(file)
    }

    async fn upload(
        &self,
        window: Option<Window>,
        id: u32,
        url: &str,
        header: HeaderMap,
        image_path: &Path,
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
            window.as_ref(),
            url,
            header,
            id,
            &self.file_part_name,
            filename,
            UploadFile::new(file, &self.file_kind),
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ManagerKind {
    API,
    Chevereto,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum ManagerCode {
    Smms,
    Imgse,
    Imgtg,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ManagerItem {
    pub key: ManagerCode,
    pub name: &'static str,
    pub index: &'static str,
    pub r#type: ManagerKind,
}

impl ManagerCode {
    pub fn name(&self) -> &'static str {
        match self {
            ManagerCode::Smms => "sm.ms",
            ManagerCode::Imgse => "imgse.com",
            ManagerCode::Imgtg => "imgtg.com",
        }
    }

    pub fn index(&self) -> &'static str {
        match self {
            ManagerCode::Smms => "https://sm.ms",
            ManagerCode::Imgse => "https://imgse.com",
            ManagerCode::Imgtg => "https://imgtg.com",
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
