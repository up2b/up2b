use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{manager::api::SerdeValueParser, Error, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DeleteKeyKind {
    Path,
    Query { key: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DeleteMethod {
    Get {
        kind: DeleteKeyKind,
    },
    Delete {
        kind: DeleteKeyKind,
    },
    Post {
        body: Map<String, Value>,
        key: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DeleteResponseController {
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
    pub(super) async fn parse(&self, response: Response) -> Result<()> {
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

                let value = json.get_value_by_keys(&key);

                if value.is_null() {
                    return Err(Error::Other("无法获取删除状态值".to_owned()));
                }

                debug!("删除状态：{}", value);

                if &value != should_be {
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

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Delete {
    pub(super) path: String,
    pub(super) method: DeleteMethod,
    pub(super) controller: DeleteResponseController,
}

impl Delete {
    pub fn new(url: &str, method: DeleteMethod, controller: DeleteResponseController) -> Self {
        Self {
            path: url.to_owned(),
            method,
            controller,
        }
    }
}
