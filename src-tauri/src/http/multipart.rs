use std::sync::{Arc, Mutex};

use futures_util::TryStreamExt;
use read_progress_stream::ReadProgressStream;
use reqwest::{multipart::Part, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, WebviewWindow};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::Up2bResult;

use super::ProgressPayload;

pub async fn file_to_body(
    id: u32,
    window: Arc<Mutex<WebviewWindow>>,
    file: File,
) -> Up2bResult<reqwest::Body> {
    let file_size = file.metadata().await?.len();
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|r| r.freeze());

    Ok(reqwest::Body::wrap_stream(ReadProgressStream::new(
        stream,
        Box::new(move |_, progress| {
            let _ = window.lock().unwrap().emit(
                "upload://progress",
                ProgressPayload {
                    id,
                    progress,
                    total: file_size,
                },
            );
        }),
    )))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum FileKind {
    Stream,
    Buffer,
}

pub struct UploadFile<'r> {
    file: File,
    kind: &'r FileKind,
}

impl<'r> UploadFile<'r> {
    pub fn new(file: File, kind: &'r FileKind) -> Self {
        Self { file, kind }
    }
}

pub async fn upload(
    request_builder: RequestBuilder,
    window: Option<&WebviewWindow>,
    id: u32,
    part_name: &str,
    filename: &str,
    mut upload_file: UploadFile<'_>,
    mime_type: &str,
    texts: Option<&[(&str, &str)]>,
) -> Up2bResult<Response> {
    let file_part = match &window {
        None => Part::stream(upload_file.file),
        Some(w) => match upload_file.kind {
            FileKind::Buffer => {
                let mut buf = Vec::new();
                upload_file.file.read_to_end(&mut buf).await?;
                Part::stream(buf)
            }
            FileKind::Stream => {
                let body = file_to_body(
                    id,
                    Arc::new(Mutex::new(w.to_owned().clone())),
                    upload_file.file,
                )
                .await?;
                Part::stream(body)
            }
        },
    }
    .file_name(filename.to_owned())
    .mime_str(mime_type)?;

    let mut form = reqwest::multipart::Form::new().part(part_name.to_owned(), file_part);

    if let Some(text_parts) = texts {
        for (name, value) in text_parts.iter() {
            form = form.text::<String, String>(name.to_string(), value.to_string());
        }
    }

    let resp = request_builder.multipart(form).send().await?;

    Ok(resp)
}
