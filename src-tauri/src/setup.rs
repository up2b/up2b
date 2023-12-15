use std::{collections::HashMap, path::Path};

use tauri::{
    api::cli::{ArgData, Matches, SubcommandMatches},
    App, Result, WindowBuilder, WindowUrl,
};

use crate::{manager::UploadResult, using_manager};

fn new_window(app: &App) {
    let builder = WindowBuilder::new(
        app,
        "main",
        #[cfg(debug_assertions)]
        WindowUrl::External("http://localhost:1420".parse().unwrap()),
        #[cfg(not(debug_assertions))]
        WindowUrl::App("../dist".into()),
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

    #[cfg(target_os = "windows")]
    {
        use window_shadows::set_shadow;

        let window = builder.transparent(true).build().unwrap();
        set_shadow(&window, true).unwrap();
    }

    #[cfg(target_os = "linux")]
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
    match app.get_cli_matches() {
        Ok(matches) => match parse_cli_matches(matches) {
            RunningMode::Cli => app.handle().exit(0), // 不退出的话会一直阻塞在主线程循环里
            RunningMode::Windows => new_window(app),
        },
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    }

    Ok(())
}
