use std::path::Path;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use tauri::Window;

#[cfg(feature = "compress")]
use super::CompressedFormat;
use crate::http::multipart::FileKind;
use crate::{error::Error, Result};

use super::{
    AllowedImageFormat, BaseManager, DeleteError, DeleteResponse, Extra, ImageItem, Manage,
    UploadResult,
};

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
    inner: BaseManager,
    token: String,
}

impl SmMs {
    pub fn new(token: String) -> Self {
        let manager = BaseManager::new(
            "sm.ms",
            "https://smms.app/api/v2/",
            5 * 1024 * 1024,
            "smfile",
            FileKind::Stream,
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
        SmMs {
            inner: manager,
            token,
        }
    }

    fn url(&self, path: &str) -> String {
        self.inner.base_url.to_owned() + path
    }

    fn header(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        header.insert("Authorization", self.token.parse().unwrap());

        header
    }

    async fn upload_history(&self) -> Result<HistoryResponse> {
        trace!("getting upload history");

        let url = self.url("upload_history");

        let response = self.inner.get(&url, self.header()).await?.json().await?;

        info!("got upload history response: {:?}", response);

        Ok(response)
    }

    async fn delete_image_by_url(&self, hash: &str) -> Result<SmDeleteResponse> {
        trace!("deleting an image");

        let url = self.url(&("delete/".to_owned() + hash));

        let response = self.inner.get(&url, self.header()).await?.json().await?;

        info!("successfully deleted the image: hash={}", hash);

        Ok(response)
    }

    async fn upload(
        &self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
    ) -> Result<SmUploadResponse> {
        trace!("uploading an image");

        let url = self.url("upload");

        let response = self
            .inner
            .upload(window, id, &url, self.header(), image_path, None)
            .await?;

        match response.json::<SmUploadResponse>().await {
            Ok(r) => {
                info!("successfully uploaded the image: {:?}", image_path);

                Ok(r)
            }
            Err(e) => {
                error!("反序例化时出错：{}", e);
                Err(Error::Reqeust(e))
            }
        }
    }
}

#[async_trait]
impl Manage for SmMs {
    fn allowed_formats(&self) -> Vec<AllowedImageFormat> {
        self.inner.allowed_formats.clone()
    }

    fn support_stream(&self) -> bool {
        true
    }

    async fn verify(&self) -> Result<Option<Extra>> {
        // TODO: api 类型的图床的 token 验证以后再实现
        Ok(None)
    }

    async fn get_all_images(&self) -> Result<Vec<ImageItem>> {
        let response = self.upload_history().await?;

        Ok(response
            .data
            .iter()
            .map(|item| ImageItem {
                url: item.url.clone(),
                deleted_id: item.hash.clone(),
                thumb: None,
            })
            .collect())
    }

    async fn delete_image(&self, id: &str) -> Result<DeleteResponse> {
        let response = self.delete_image_by_url(id).await?;

        Ok(DeleteResponse {
            success: response.success,
            error: if response.success {
                None
            } else {
                Some(DeleteError::Other(response.message))
            },
        })
    }

    async fn upload_image(
        &self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
    ) -> UploadResult {
        let resp = match self.upload(window, id, image_path).await {
            Ok(r) => r,
            Err(e) => {
                return UploadResult::Error {
                    code: e.as_string(),
                    detail: e,
                }
            }
        };

        match resp.data {
            None => {
                return UploadResult::Error {
                    detail: Error::Other(resp.message),
                    code: resp.code,
                };
            }
            Some(data) => UploadResult::Response(ImageItem {
                url: data.url,
                deleted_id: data.hash,
                thumb: None,
            }),
        }
    }
}
