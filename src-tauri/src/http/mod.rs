use serde::Serialize;

pub mod json;
pub mod multipart;

#[derive(Clone, Serialize)]
struct ProgressPayload {
    id: u32,
    progress: u64,
    total: u64,
}
