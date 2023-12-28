// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod http;
mod logger;
mod manager;
mod setup;
mod util;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate simplelog;

use error::AuthConfigError;
use manager::api::Api;
use manager::smms::SMMS_API;
use manager::ManagerItem;
use simplelog::CombinedLogger;
#[cfg(not(debug_assertions))]
use simplelog::WriteLogger;
#[cfg(debug_assertions)]
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use std::path::PathBuf;
use tauri::Window;

use crate::config::{write_config, Config, ManagerAuthConfigKind, APP_CONFIG_DIR, CONFIG};
use crate::logger::{log_level, logger_config};
use crate::manager::{
    use_manager, AllowedImageFormat, DeleteResponse, Extra, ImageItem, Manage, UploadResult,
    MANAGERS,
};

pub use crate::error::{ConfigError, Error, Result};
use crate::manager::ManagerCode;

async fn using_manager() -> Result<Box<dyn Manage>> {
    let config = match CONFIG.read().await.as_ref() {
        None => return Err(Error::Config(ConfigError::NotFound)),
        Some(c) => c.clone(),
    };

    let using = config.using();
    let auth_config = config.get_auth_config(&using);

    if let Some(c) = auth_config {
        return use_manager(&using, c);
    }

    Err(Error::AuthConfig(AuthConfigError::Null(using.clone())))
}

#[tauri::command]
async fn get_managers() -> Vec<ManagerItem> {
    let conf = CONFIG.read().await;
    let conf = conf.clone();

    let mut managers = MANAGERS.to_vec();

    if let Some(c) = conf {
        let auth_config = c.auth_config();
        for key in auth_config.keys() {
            if let ManagerCode::Custom(_) = key {
                managers.push(key.clone().to_manager_item());
            }
        }
    }

    managers
}

#[tauri::command]
async fn get_all_images() -> Result<Vec<ImageItem>> {
    trace!("获取图片列表");

    let uploader = using_manager().await?;

    Ok(uploader.get_all_images().await?)
}

#[tauri::command]
async fn delete_image(delete_id: String) -> Result<DeleteResponse> {
    trace!("删除图片：{}", delete_id);
    let uploader = using_manager().await?;

    Ok(uploader.delete_image(&delete_id).await?)
}

#[tauri::command]
async fn upload_image(window: Window, image_path: PathBuf) -> Result<UploadResult> {
    trace!("上传图片 {image_path:?}");

    let uploader = using_manager().await?;

    Ok(uploader.upload_image(Some(window), 1, &image_path).await)
}

#[tauri::command]
async fn verify(image_bed: ManagerCode, config: ManagerAuthConfigKind) -> Result<Option<Extra>> {
    let uploader = use_manager(&image_bed, &config)?;

    uploader.verify().await
}

#[tauri::command]
async fn get_config() -> Option<Config> {
    let conf = CONFIG.read().await;

    conf.clone()
}

#[tauri::command]
async fn smms_config() -> Api {
    // 如果没有配置文件，将 smms 示例添加到配置中
    SMMS_API.clone()
}

#[tauri::command]
async fn update_config(config: Config) -> Result<()> {
    write_config(&config)?;

    let mut c = CONFIG.write().await;
    *c = Some(config);

    Ok(())
}

#[tauri::command]
fn compress_state() -> bool {
    cfg!(feature = "compress")
}

#[tauri::command]
async fn support_stream() -> Result<bool> {
    let uploader = using_manager().await?;

    Ok(uploader.support_stream())
}

#[tauri::command]
async fn allowed_formats() -> Result<Vec<AllowedImageFormat>> {
    let uploader = using_manager().await?;

    Ok(uploader.allowed_formats())
}

#[tauri::command]
async fn get_using_image_bed() -> ManagerCode {
    let read_guard = CONFIG.read().await;
    let config = read_guard.as_ref();
    config.unwrap().using().clone()
}

#[tauri::command]
async fn toggle_manager(manager_code: ManagerCode) -> Result<()> {
    let mut write_guard = CONFIG.write().await;

    if let Some(mut config) = write_guard.take() {
        config.set_using(manager_code);
        write_config(&config)?;
        *write_guard = Some(config);
    }

    Ok(())
}

#[tauri::command]
async fn automatic_compression() -> bool {
    CONFIG
        .read()
        .await
        .as_ref()
        .unwrap()
        .automatic_compression()
}

#[tauri::command]
async fn new_custom_manager(manager_code: ManagerCode, auth_config: ManagerAuthConfigKind) {
    println!("manager: {} config: {:?}", manager_code, auth_config);
}

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![
        #[cfg(debug_assertions)]
        TermLogger::new(
            log_level(),
            logger_config(true),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        #[cfg(not(debug_assertions))]
        WriteLogger::new(
            log_level(),
            logger_config(true),
            std::fs::File::create(APP_CONFIG_DIR.join("up2b.log")).unwrap(),
        ),
    ])
    .unwrap();

    info!("配置文件路径：{:?}", *APP_CONFIG_DIR);

    #[cfg(feature = "compress")]
    {
        use crate::util::image::compress::TEMP_DIR;
        use std::fs::create_dir;

        if !TEMP_DIR.exists() {
            create_dir(TEMP_DIR.as_path()).unwrap();
        }
    }

    let builder = tauri::Builder::default().setup(|app| match setup::setup(&app) {
        Err(_) => Ok(app.handle().exit(1)),
        _ => Ok(()),
    });

    builder
        .invoke_handler(tauri::generate_handler![
            get_all_images,
            delete_image,
            upload_image,
            get_config,
            update_config,
            compress_state,
            verify,
            get_using_image_bed,
            support_stream,
            allowed_formats,
            toggle_manager,
            automatic_compression,
            smms_config,
            get_managers,
            new_custom_manager,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
