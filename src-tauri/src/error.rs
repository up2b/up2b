use std::time::SystemTimeError;

use serde::{Serialize, Serializer};

use crate::manager::ManagerCode;

pub type Up2bResult<T> = std::result::Result<T, Up2bError>;

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
pub enum Up2bError {
    #[error(transparent)]
    Cli(#[from] tauri_plugin_cli::Error),
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

    #[error(transparent)]
    Chevereto(#[from] CheveretoError), // 使用 Chevereto 写的图床，有 imgse、imgtg

    #[error(transparent)]
    Header(#[from] HeaderError),

    #[error("connection closed before message completed. Should retry.")]
    ConnectionClosedBeforeMessageCompleted,

    #[error("{} 已存在", .0.name())]
    CustomUnique(ManagerCode),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error(transparent)]
    Upload(#[from] UploadError),

    #[error("响应体中未找到 key：{0}")]
    KeyNotFound(String),
    #[error("响应体中 key[{0}] 对应的类型不匹配")]
    KeyNotMatch(String),

    #[error(transparent)]
    Path(#[from] PathError),

    #[error(transparent)]
    Git(#[from] GitError),
}

impl Up2bError {
    pub fn as_string(&self) -> String {
        match self {
            Self::OverSize(_, _, _, _) => "OVER_SIZE".to_owned(),
            Self::Path(e) => e.as_str().to_owned(),
            Self::Upload(e) => match e {
                UploadError::Repeat(_) => "REPEATED".to_owned(),
                _ => "UNKOWN".to_owned(),
            },
            _ => "UNKOWN".to_owned(),
        }
    }
}

impl Serialize for Up2bError {
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
    #[error(transparent)]
    InvalidValue(reqwest::header::InvalidHeaderValue),
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("{0}")]
    Error(String),
    #[error("{0}")]
    Repeat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("路径不存在")]
    NotExists,
    #[error("路径不是文件")]
    NotFile,
}

impl PathError {
    pub fn as_str(&self) -> &str {
        match self {
            PathError::NotExists => "NOT_EXISTS",
            PathError::NotFile => "NOT_FILE",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("资源不存在")]
    NotFound,
    #[error("{0}")]
    Other(String),
}

impl GitError {
    pub fn as_str(&self) -> &str {
        match self {
            GitError::NotFound => "NOT_FOUND",
            GitError::Other(s) => &s,
        }
    }
}

impl Into<GitError> for String {
    fn into(self) -> GitError {
        if self == "Not Found" {
            return GitError::NotFound;
        }

        GitError::Other(self)
    }
}
