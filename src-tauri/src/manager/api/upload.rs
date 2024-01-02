use regex::Regex;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    error::UploadError,
    http::multipart::FileKind,
    manager::{AllowedImageFormat, ImageItem},
    Error, Result,
};

#[cfg(feature = "compress")]
use crate::manager::CompressedFormat;

use super::SerdeValueParser;

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
pub struct UploadResponseErrorController {
    /// 重复的错误消息键
    key: String,
    /// 错误信息中存在的上传重复链接的正则表达式，可选
    repeated_regex: Option<String>,
}

impl UploadResponseErrorController {
    pub fn new<S: Into<String>, O: Into<Option<String>>>(key: S, repeat_regex: O) -> Self {
        Self {
            key: key.into(),
            repeated_regex: repeat_regex.into(),
        }
    }

    pub fn parse(&self, json: &Value) -> Error {
        let error = json.get_value_by_keys(&self.key);
        let error_message = match error.as_str() {
            Some(s) => s,
            None => return Error::KeyNotFound(self.key.to_owned()),
        };

        // 无 repeat 正则时直接返回错误
        match &self.repeated_regex {
            Some(r) => {
                // regex 无法序列化，只能每次使用时初始化
                let regex = match Regex::new(&r) {
                    Ok(r) => r,
                    Err(e) => return e.into(),
                };

                if let Some(captures) = regex.captures(error_message) {
                    match captures.get(1) {
                        None => return UploadError::Error(error_message.to_owned()).into(),
                        Some(v) => {
                            return UploadError::Repeat(v.as_str().to_owned()).into();
                        }
                    }
                }

                UploadError::Error(error_message.to_owned()).into()
            }
            None => UploadError::Error(error_message.to_owned()).into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadResponseSuccuessController {
    image_url_key: String,
    /// 有的图床不提供缩略图
    thumb_key: Option<String>,
    deleted_id_key: String,
}

impl UploadResponseSuccuessController {
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

    fn parse(&self, json: &Value) -> Result<ImageItem> {
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
pub struct UploadResponseStatus {
    key: String,
    value: Value,
}

impl UploadResponseStatus {
    pub fn new<S: Into<String>, V: Into<Value>>(key: S, value: V) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadResponseController {
    status: UploadResponseStatus,
    error: UploadResponseErrorController,
    success: UploadResponseSuccuessController,
}

impl UploadResponseController {
    pub fn new(
        status: UploadResponseStatus,
        error: UploadResponseErrorController,
        success: UploadResponseSuccuessController,
    ) -> Self {
        Self {
            status,
            error,
            success,
        }
    }

    pub async fn parse(&self, response: Response) -> Result<ImageItem> {
        let json: Value = response.json().await?;

        debug!("响应体：{}", json);

        let status = json.get_value_by_keys(&self.status.key);

        debug!("status: {}, {}", self.status.value, status);

        // 上传失败
        if status != self.status.value {
            return Err(self.error.parse(&json));
        }

        self.success.parse(&json)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upload {
    pub(super) path: String,
    pub(super) max_size: u8,
    pub(super) allowed_formats: Vec<AllowedImageFormat>,
    #[cfg(feature = "compress")]
    pub(super) compressed_format: CompressedFormat,
    pub(super) content_type: UploadContentType,
    /// 请求体中图片之外的其他部分
    other_body: Option<Map<String, Value>>,
    pub(super) controller: UploadResponseController,
    pub(super) timeout: u64,
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
