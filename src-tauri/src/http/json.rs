use std::{
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::stream::Stream;
use read_progress_stream::ReadProgressStream;
use reqwest::{header::CONTENT_TYPE, RequestBuilder, Response};
use serde_json::Value;
use tauri::Window;

use crate::Result;

use super::ProgressPayload;

struct BytesStream {
    data: Vec<u8>,
    chunk_size: usize,
    cursor: usize,
}

impl BytesStream {
    fn new(data: Vec<u8>, chunk_size: usize) -> Self {
        Self {
            data,
            chunk_size,
            cursor: 0,
        }
    }
}
impl Stream for BytesStream {
    type Item = std::result::Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let len = self.data.len();
        if self.cursor >= len {
            Poll::Ready(None)
        } else {
            let data = std::mem::replace(&mut self.data, Vec::new());

            let end = (self.cursor + self.chunk_size).min(data.len());

            let chunk = &data[self.cursor..end].to_vec();

            self.cursor = end;

            self.data = data;

            let bytes = Bytes::copy_from_slice(chunk);

            Poll::Ready(Some(Ok(bytes)))
        }
    }
}

pub async fn bytes_to_body(id: u32, window: Window, bytes: Vec<u8>) -> Result<reqwest::Body> {
    let size = bytes.len();

    let window = Mutex::new(window);

    Ok(reqwest::Body::wrap_stream(ReadProgressStream::new(
        BytesStream::new(bytes, 8 * 1024),
        Box::new(move |_, progress| {
            let _ = window.lock().unwrap().emit(
                "upload://progress",
                ProgressPayload {
                    id,
                    progress,
                    total: size as u64,
                },
            );
        }),
    )))
}

pub async fn upload(
    request_builder: RequestBuilder,
    window: Option<&Window>,
    body: Value,
    id: u32,
) -> Result<Response> {
    let builder = match window {
        None => request_builder.json(&body),
        Some(w) => {
            let stream = bytes_to_body(id, w.clone(), serde_json::to_vec(&body)?).await?;

            request_builder
                .body(stream)
                .header(CONTENT_TYPE, "application/json")
        }
    };

    let resp = builder.send().await?;

    Ok(resp)
}
