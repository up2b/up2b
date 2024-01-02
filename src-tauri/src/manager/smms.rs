use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Window;

use crate::Result;
use crate::{http::multipart::FileKind, manager::api::UploadResponseStatus};

use super::api::{
    Api, AuthMethod, BaseApiManager, Delete, DeleteKeyKind, DeleteMethod, List, ListRequestMethod,
    ListResponseController, Upload, UploadResponseController, UploadResponseErrorController,
    UploadResponseSuccuessController,
};
#[cfg(feature = "compress")]
use super::CompressedFormat;
use super::{
    AllowedImageFormat, BaseManager, DeleteError, DeleteResponse, Extra, ImageItem, Manage,
    UploadResult,
};

lazy_static! {
    pub static ref SMMS_API: Api = {
        let status = UploadResponseStatus::new("success", true);
        let error = UploadResponseErrorController::new(
            "message",
            "^Image upload repeated limit, this image exists at: (.+?)$".to_owned(),
        );
        let success = UploadResponseSuccuessController::new("data.url", None, "data.hash");
        let controller = UploadResponseController::new(status, error, success);

        let upload = Upload::new(
            "/upload",
            5,
            vec![
                AllowedImageFormat::Jpeg,
                AllowedImageFormat::Png,
                AllowedImageFormat::Gif,
                AllowedImageFormat::Bmp,
                AllowedImageFormat::Webp,
            ],
            #[cfg(feature = "compress")]
            CompressedFormat::WEBP,
            super::api::UploadContentType::Multipart {
                file_part_name: "smfile".to_owned(),
                file_kind: FileKind::Stream,
            },
            None,
            controller,
            5,
        );

        let list = List::new(
            "/upload_history",
            ListResponseController::new("data", "url", "hash", None),
            ListRequestMethod::Get,
        );

        let delete = Delete::new(
            "/delete/",
            DeleteMethod::Get {
                kind: DeleteKeyKind::Path,
            },
            super::api::DeleteResponseController::Json {
                key: "success".to_owned(),
                should_be: Value::Bool(true),
                message_key: Some("message".to_owned()),
            },
        );

        let auth_method = AuthMethod::Header {
            key: None,
            prefix: None,
        };

        Api::new("https://smms.app/api/v2", auth_method, upload, list, delete)
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryDataItem {
    width: u16,
    height: u16,
    filename: String,
    storename: String,
    size: u64,
    path: String,
    hash: String,
    created_at: String,
    url: String,
    delete: String,
    page: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryResponse {
    success: bool,
    code: String,
    message: String,
    data: Vec<HistoryDataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SmDeleteResponse {
    success: bool,
    code: String,
    message: String,
    #[serde(rename = "RequestId")]
    request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SmUploadData {
    file_id: u32,
    width: u32,
    height: u32,
    filename: String,
    storename: String,
    size: u32,
    path: String,
    hash: String,
    url: String,
    delete: String,
    page: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SmUploadResponse {
    success: bool,
    code: String,
    message: String,
    #[serde(rename = "RequestId")]
    request_id: String,
    data: Option<SmUploadData>,
}

#[derive(Debug)]
pub struct SmMs {
    inner: BaseApiManager,
}

impl SmMs {
    pub fn new(token: String) -> Self {
        let manager = BaseManager::new(
            "sm.ms",
            5,
            vec![
                AllowedImageFormat::Jpeg,
                AllowedImageFormat::Png,
                AllowedImageFormat::Gif,
                AllowedImageFormat::Bmp,
                AllowedImageFormat::Webp,
            ],
            #[cfg(feature = "compress")]
            CompressedFormat::WEBP,
        );

        let inner = BaseApiManager::new(manager, token.clone(), &SMMS_API);
        SmMs { inner }
    }
}

#[async_trait]
impl Manage for SmMs {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat> {
        self.inner.allowed_formats()
    }

    fn support_stream(&self) -> bool {
        true
    }

    async fn verify(&self) -> Result<Option<Extra>> {
        // TODO: api 类型的图床的 token 验证以后再实现
        Ok(None)
    }

    async fn get_all_images(&self) -> Result<Vec<ImageItem>> {
        self.inner.list().await
    }

    async fn delete_image(&self, id: &str) -> Result<DeleteResponse> {
        match self.inner.delete(id).await {
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
        match self.inner.upload(window, id, image_path, None).await {
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
