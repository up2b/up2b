use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;
use tauri::Window;

use crate::http::multipart::FileKind;
use crate::manager::{
    AllowedImageFormat, DeleteResponse, Extra, ImageItem, Manage, ManagerCode, UploadResult,
};
use crate::Result;

use super::Chevereto;
#[cfg(feature = "compress")]
use super::CompressedFormat;

#[derive(Debug)]
pub struct Imgse {
    inner: Chevereto,
}

impl Imgse {
    pub fn new<S: Into<String>>(
        username: S,
        password: S,
        extra: Option<&HashMap<String, String>>,
    ) -> Self {
        Self {
            inner: Chevereto::new(
                ManagerCode::Imgse,
                "imgse.com",
                "https://imgse.com/",
                username,
                password,
                10 * 1024 * 1024,
                FileKind::Buffer,
                vec![
                    AllowedImageFormat::Jpeg,
                    AllowedImageFormat::Png,
                    AllowedImageFormat::Gif,
                ],
                extra,
                #[cfg(feature = "compress")]
                CompressedFormat::JPEG,
            ),
        }
    }
}

#[async_trait]
impl Manage for Imgse {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat> {
        self.inner.manager.allowed_formats.to_owned()
    }

    fn support_stream(&self) -> bool {
        self.inner.manager.file_kind == FileKind::Stream
    }

    async fn verify(&self) -> Result<Option<Extra>> {
        let mut inner = self.inner.clone();
        Ok(Some(inner.login().await?))
    }

    async fn get_all_images(&self) -> Result<Vec<ImageItem>> {
        self.inner.get_user_images().await
    }

    async fn delete_image(&self, id: &str) -> Result<DeleteResponse> {
        let mut inner = self.inner.clone();
        inner.delete_image_by_id(id, 0).await
    }

    async fn upload_image(
        &self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
    ) -> UploadResult {
        let mut inner = self.inner.clone();

        let resp = match inner.upload(window.clone(), id, image_path, 0).await {
            Ok(r) => r,
            Err(e) => {
                return UploadResult::Error {
                    code: e.as_string(),
                    detail: e,
                }
            }
        };

        UploadResult::Response(ImageItem {
            url: resp.image.url,
            deleted_id: resp.image.name,
            thumb: Some(resp.image.thumb.url),
        })
    }
}
