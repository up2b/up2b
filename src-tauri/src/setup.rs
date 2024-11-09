use std::{collections::HashMap, path::Path};

use tauri::{App, Result, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_cli::{ArgData, CliExt, Matches, SubcommandMatches};

use crate::{manager::UploadResult, using_manager};

fn new_window(app: &App) {
    let builder = WebviewWindowBuilder::new(
        app,
        "main",
        #[cfg(debug_assertions)]
        WebviewUrl::External("http://localhost:1420".parse().unwrap()),
        #[cfg(not(debug_assertions))]
        WebviewUrl::App("../dist".into()),
    )
    .decorations(cfg!(not(target_os = "windows")))
    .title("up2b")
    .min_inner_size(760.0, 600.0)
    .inner_size(760.0, 600.0);

    #[cfg(target_os = "macos")]
    {
        builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .build()
            .unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    builder.build().unwrap();
}

fn upload(command: Box<SubcommandMatches>) {
    let images: Vec<String> = command
        .matches
        .args
        .get("images")
        .unwrap()
        .value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();

    tokio::task::block_in_place(|| {
        tauri::async_runtime::block_on(async move {
            let uploader = using_manager().await.unwrap();

            for image in images.iter() {
                let result = uploader.upload_image(None, 0, Path::new(image)).await;
                match result {
                    UploadResult::Response(r) => println!("{}", r.url),
                    UploadResult::Error { detail, .. } => println!("{}", detail),
                }
            }
        })
    });
}

enum RunningMode {
    Cli,
    Windows,
}

// 本程序中只允许传入一个可选参数，args 的长度只能为 1，只打印 ArgData 即可
fn parse_cli_args(args: HashMap<String, ArgData>) {
    let data = args.values().next().unwrap();

    let s = data.value.as_str().unwrap();
    println!("{s}");
}

fn parse_cli_matches(matches: Matches) -> RunningMode {
    if matches.args.len() > 0 {
        debug!("cli args: {:?}", matches.args);
        parse_cli_args(matches.args);
        return RunningMode::Cli;
    }

    if let Some(subcommond) = matches.subcommand {
        debug!("cli command: {:?}", subcommond);
        // 处理 upload 命令
        if subcommond.name == "upload" {
            upload(subcommond);
            return RunningMode::Cli;
        }
    }

    RunningMode::Windows
}

pub(crate) fn setup(app: &App) -> Result<()> {
    let matches = app
        .cli()
        .matches()
        .map_err(|e| {
            error!("{}", e);
            e
        })
        .unwrap();

    match parse_cli_matches(matches) {
        RunningMode::Cli => app.handle().exit(0), // 不退出的话会一直阻塞在主线程循环里
        RunningMode::Windows => new_window(app),
    }

    Ok(())
}
