#[cfg(feature = "compress")]
pub mod compress;

pub fn guess_mime_type_by_ext(filename: &str) -> String {
    let ext = filename.rsplitn(2, '.').last();

    match ext {
        None => return "image/jpeg".to_owned(),
        Some(s) => {
            let lower = s.to_lowercase();
            match lower.as_str() {
                "jpg" | "jpeg" => "image/jpeg".to_owned(),
                _ => "image/".to_owned() + s,
            }
        }
    }
}
