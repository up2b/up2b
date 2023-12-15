use std::path::PathBuf;

use image::io::Reader;

use crate::{util::time::now, Error};

use {
    crate::{manager::CompressedFormat, Result},
    image::{
        imageops::{self, FilterType},
        DynamicImage,
    },
    serde::Serialize,
    std::io::Cursor,
    tauri::Window,
    tokio::{fs::File, io::AsyncReadExt},
};

lazy_static! {
    pub static ref TEMP_DIR: PathBuf = std::env::temp_dir().join("up2b-".to_owned() + &now().unwrap().as_secs().to_string()); // 不能重名，加时间后缀
}

fn compress_to_jpeg(img: DynamicImage, max_size: u64, file_size: u64) -> Result<DynamicImage> {
    let quality: u8 = (max_size * 100 / file_size) as u8;
    let mut compressed_img = DynamicImage::new_rgba8(img.width(), img.height());

    // 将原始图片的像素拷贝到压缩图片中
    imageops::replace(&mut compressed_img, &img.to_rgba8(), 0, 0);

    // 获取压缩后的 JPEG 数据
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    compressed_img.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(quality))?;

    Ok(image::load_from_memory(&buffer)?)
}

const COMPRESS_EVENT_NAME: &str = "upload://compress";

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
enum CompressEvent {
    No,
    Start,
    End {
        filename: String,
        original: u64,
        compressed: u64,
    },
}

pub async fn compress(
    window: Option<&Window>,
    max_size: u64,
    file_size: u64,
    filename: &str,
    mut image_file: File,
    compressed_format: &CompressedFormat,
) -> Result<File> {
    if max_size >= file_size {
        match window {
            Some(w) =>
            // 通知前端不需要压缩
            {
                w.emit(COMPRESS_EVENT_NAME, &CompressEvent::No)?;
            }
            None => {}
        }
        return Ok(image_file);
    }

    match window {
        None => {}
        Some(w) => {
            w.emit(COMPRESS_EVENT_NAME, &CompressEvent::Start)?;
        }
    }
    info!("图片尺寸超过图床限制，正在压缩图片。");

    let mut buf = vec![];
    image_file.read_to_end(&mut buf).await?;
    trace!("已读取图片到缓存");

    #[cfg(not(feature = "no-limits"))]
    let reader = Reader::new(Cursor::new(buf)).with_guessed_format()?;

    #[cfg(feature = "no-limits")]
    let reader = {
        let mut r = Reader::new(Cursor::new(buf)).with_guessed_format()?;

        r.no_limits();

        r
    };

    let img = match reader.decode() {
        Ok(p) => p,
        Err(e) => {
            error!("读取图片失败：{}", e);
            return Err(Error::Image(e));
        }
    };

    debug!("要保存的图片格式：{:?}", compressed_format);

    let filename_without_ext = filename.rsplitn(2, ".").collect::<Vec<&str>>()[1];

    let path = match compressed_format {
        CompressedFormat::JPEG => {
            let img = compress_to_jpeg(img, max_size, file_size)?;
            let p = TEMP_DIR.join(filename_without_ext.to_owned() + ".jpeg");
            img.save(&p)?;
            p
        }
        CompressedFormat::WEBP => {
            let scale = (file_size as f64 / max_size as f64).sqrt().ceil();
            // 设置目标尺寸
            let target_width = (img.width() as f64 / scale).floor() as u32;
            let target_height = (img.height() as f64 / scale).floor() as u32;
            debug!("压缩后的图片尺寸：{} x {}", target_width, target_height);

            // 缩放图像
            let img = img.resize_exact(target_width, target_height, FilterType::Lanczos3);

            let p = TEMP_DIR.join(filename_without_ext.to_owned() + ".webp");
            img.save(&p)?;
            p
        }
    };

    let file = File::open(&path).await?;

    let size = file.metadata().await?.len();

    debug!("压缩图片已保存到本地：{:?}，压缩后体积：{}", path, size);

    match window {
        None => {}
        Some(w) => {
            w.emit(
                COMPRESS_EVENT_NAME,
                &CompressEvent::End {
                    filename: filename.into(),
                    original: file_size,
                    compressed: size,
                },
            )?;
        }
    }

    Ok(file)
}
