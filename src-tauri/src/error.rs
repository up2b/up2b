use std::time::SystemTimeError;

use serde::{Serialize, Serializer};

use crate::manager::ManagerCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config: not found")]
    NotFound,
    #[error("config: {0} is null")]
    IsNull(String),
    #[error("config: {0} type is error")]
    Type(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Reqeust(#[from] reqwest::Error),
    #[error("状态码错误：{0}")]
    Status(reqwest::StatusCode),
    #[cfg(feature = "compress")]
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("图床 {0} 不可上传此体积图片：path={1}, size={3}M > {2}M")]
    OverSize(String, String, u64, u64),
    #[error(transparent)]
    Time(#[from] SystemTimeError),

    #[error("用户名或密码错误")]
    Auth,

    #[error(transparent)]
    AuthConfig(#[from] AuthConfigError),

    #[error(transparent)]
    Proxy(#[from] ProxyError),

    #[error("{0}")]
    Other(String),

    #[error("{0}")]
    Chevereto(#[from] CheveretoError), // 使用 Chevereto 写的图床，有 imgse、imgtg

    #[error(transparent)]
    Header(#[from] HeaderError),

    #[error("connection closed before message completed. Should retry.")]
    ConnectionClosedBeforeMessageCompleted,

    #[error("{} 已存在", .0.name())]
    CustomUnique(ManagerCode),
}

impl Error {
    pub fn as_string(&self) -> String {
        match self {
            Self::OverSize(_, _, _, _) => "OVER_SIZE".to_owned(),
            _ => "UNKOWN".to_owned(),
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CheveretoError {
    #[error("auth_token 失效")]
    AuthToken,
    #[error("图片不存在")]
    InvalidContentOwnerRequest,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("代理配置为空")]
    Null,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthConfigError {
    #[error("{0} 配置为空")]
    Null(ManagerCode),
}

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error(transparent)]
    InvalidName(reqwest::header::InvalidHeaderName),
}
