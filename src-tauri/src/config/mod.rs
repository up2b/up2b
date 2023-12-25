use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::manager::api::Api;
use crate::manager::smms::SMMS_API;
use crate::ManagerCode;

lazy_static! {
    pub static ref APP_CONFIG_DIR: PathBuf = {
        let config_dir = dirs::config_dir().unwrap();

        let app_config_dir = config_dir.join("up2b");

        if !app_config_dir.exists() {
            fs::create_dir(&app_config_dir).unwrap();
        }

        app_config_dir
    };
    static ref CONFIG_FILE: PathBuf = APP_CONFIG_DIR.join("config.toml");
    pub static ref CONFIG: RwLock<Option<Config>> = RwLock::new(match read_config() {
        Ok(c) => c,
        Err(e) => panic!("{e}"),
    });
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProxyKind {
    Http { host: String, port: u32 },
    Https { host: String, port: u32 },
    Socks5 { host: String, port: u32 },
    Socks5h { host: String, port: u32 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum ManagerAuthConfigKind {
    API {
        token: String,
        api: Api,
    },
    Git {
        token: String,
    },
    Chevereto {
        username: String,
        password: String,
        extra: Option<HashMap<String, String>>,
    },
}

fn default_image_bed() -> ManagerCode {
    ManagerCode::Smms
}

fn default_use_proxy() -> bool {
    false
}

fn default_automatic_compression() -> bool {
    false
}

/// 图床属性名应该是 ImageBedCode 的小写
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_image_bed")]
    using: ManagerCode,
    #[serde(default = "default_automatic_compression")]
    automatic_compression: bool,
    #[serde(default = "default_use_proxy")]
    use_proxy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy: Option<ProxyKind>,
    auth_config: HashMap<ManagerCode, ManagerAuthConfigKind>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            using: Default::default(),
            automatic_compression: Default::default(),
            use_proxy: Default::default(),
            proxy: None,
            auth_config: HashMap::default(),
        }
    }
}

impl Config {
    pub fn using(&self) -> &ManagerCode {
        &self.using
    }

    pub fn set_using(&mut self, manager_code: ManagerCode) {
        self.using = manager_code;
    }

    pub fn automatic_compression(&self) -> bool {
        self.automatic_compression
    }

    // pub fn enable_automatic_compression(&mut self) {
    //     if !self.automatic_compression {
    //         self.automatic_compression = true;
    //     }
    // }
    //
    // pub fn disable_automatic_compression(&mut self) {
    //     if self.automatic_compression {
    //         self.automatic_compression = false;
    //     }
    // }
    //
    // pub fn enable_proxy(&mut self) -> Result<()> {
    //     if self.use_proxy {
    //         return Ok(());
    //     }
    //
    //     if self.proxy.is_none() {
    //         return Err(Error::Proxy(ProxyError::Null));
    //     }
    //
    //     self.use_proxy = true;
    //
    //     Ok(())
    // }
    //
    // pub fn disable_proxy(&mut self) {
    //     if self.use_proxy {
    //         self.use_proxy = false;
    //     }
    // }
    //
    // pub fn proxy(&self) -> Option<&ProxyKind> {
    //     self.proxy.as_ref()
    // }
    //
    // pub fn set_proxy(&mut self, proxy: ProxyKind) {
    //     self.proxy = Some(proxy);
    // }
    //
    // pub fn auth_config(&self) -> HashMap<ManagerCode, ManagerAuthConfigKind> {
    //     self.auth_config.clone()
    // }

    pub fn get_auth_config(&self, manage_code: &ManagerCode) -> Option<&ManagerAuthConfigKind> {
        self.auth_config.get(manage_code)
    }

    pub fn insert_auth_config(&mut self, manage_code: ManagerCode, config: ManagerAuthConfigKind) {
        self.auth_config.insert(manage_code, config);
    }

    // pub fn set_auth_config(&mut self, auth_config: HashMap<ManagerCode, ManagerAuthConfigKind>) {
    //     self.auth_config = auth_config;
    // }
}

fn read_config() -> Result<Option<Config>> {
    if !CONFIG_FILE.exists() {
        return Ok(None);
    }

    let config_str = fs::read_to_string(CONFIG_FILE.to_owned())?;

    let mut config: Config = toml::from_str(&config_str)?;

    config.auth_config.insert(
        ManagerCode::Smms,
        ManagerAuthConfigKind::API {
            token: "".to_owned(),
            api: SMMS_API.clone(),
        },
    );

    Ok(Some(config))
}

pub fn write_config(config: &Config) -> Result<()> {
    trace!("write a new config");

    let config_str = toml::to_string(config)?;

    debug!("serialized config：{}", config_str);

    Ok(fs::write(CONFIG_FILE.to_owned(), config_str)?)
}
