mod imgse;
mod imgtg;

use std::{collections::HashMap, path::Path};

use async_recursion::async_recursion;
use regex::Regex;
use reqwest::{header::HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};
use tauri::Window;

use crate::{
    config::{write_config, ManagerAuthConfigKind, CONFIG},
    error::{CheveretoError, Result},
    http::multipart::FileKind,
    manager::DeleteError,
    util::time::now,
    Error,
};

#[cfg(feature = "compress")]
use super::CompressedFormat;

pub use imgse::Imgse;
pub use imgtg::Imgtg;

use super::{AllowedImageFormat, BaseManager, DeleteResponse, Extra, ImageItem, ManagerCode};

const MAX_RETRY_COUNT: u8 = 3;

#[derive(Debug, Serialize, Deserialize)]
struct HistoryDataItem {
    width: u16,
    height: u16,
    filename: String,
    storename: String,
    size: u64,
    path: String,
    hash: String,
    created_at: String,
    url: String,
    delete: String,
    page: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryResponse {
    success: bool,
    code: String,
    message: String,
    data: Vec<HistoryDataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoDeleteResponse {
    status_code: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoUploadSuccess {
    message: String,
    code: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct Thumb {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoImage {
    name: String,
    url: String,
    thumb: Thumb,
}

impl Into<CheveretoError> for CheveretoErrorDetail {
    fn into(self) -> CheveretoError {
        if self.message == "请求被拒绝 (auth_token)" {
            return CheveretoError::AuthToken;
        } else if self.message == "Invalid content owner request" {
            return CheveretoError::InvalidContentOwnerRequest;
        }

        CheveretoError::Other(self.message)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoErrorDetail {
    message: String,
    code: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoRequest {
    r#type: String,
    action: String,
    timestamp: String,
    auth_token: String,
    nsfw: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoErrorResponse {
    status_code: u16,
    status_txt: String,
    error: CheveretoErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheveretoUploadResponse {
    status_code: u8,
    status_txt: String,
    success: CheveretoUploadSuccess,
    image: CheveretoImage,
}

lazy_static! {
    static ref QUOTES: HashMap<&'static str, &'static str> = HashMap::from([
        ("%7B", "{"),
        ("%22", "\""),
        ("%3A", ":"),
        ("%2C", ","),
        ("%7D", "}"),
        ("%5C", "\\"),
        ("%2F", "/")
    ]);
}

fn unquote(text: &str) -> String {
    let mut src = text.to_string();

    for (k, v) in QUOTES.iter() {
        src = src.replace(k, v);
    }

    src
}

#[derive(Debug, Clone)]
pub struct Chevereto {
    inner: BaseManager,
    code: ManagerCode,
    file_kind: FileKind,
    file_part_name: String,
    username: String,
    password: String,
    token: Option<String>,
    cookie: Option<String>,
}

impl Chevereto {
    pub fn new<S: Into<String>>(
        code: ManagerCode,
        name: &str,
        base_url: &str,
        username: S,
        password: S,
        max_size: u8,
        file_kind: FileKind,
        allowed_formats: Vec<AllowedImageFormat>,
        timeout: u8,
        extra: Option<&HashMap<String, String>>,
        #[cfg(feature = "compress")] compressed_format: CompressedFormat,
    ) -> Self {
        let (token, cookie) = match extra {
            None => (None, None),
            Some(m) => (m.get("token"), m.get("cookie")),
        };

        let manager = BaseManager::new(
            name,
            base_url,
            max_size,
            allowed_formats,
            Some(timeout),
            #[cfg(feature = "compress")]
            compressed_format,
        );

        Self {
            inner: manager,
            file_part_name: "source".to_string(),
            file_kind,
            username: username.into(),
            password: password.into(),
            token: token.cloned(),
            cookie: cookie.cloned(),
            code,
        }
    }

    async fn get_auth_data(&self, no_cookie: bool) -> Result<Option<(String, HeaderMap)>> {
        trace!("get auth data");

        let mut headers = self.header();
        headers.insert("Accept", "text/html".parse().unwrap());
        if no_cookie {
            headers.remove("Cookie");
        }

        debug!("get auth data: request headers: {:?}", headers);

        let url = self.inner.url("login");

        let resp = self.inner.post(&url, headers).await?;

        let pattern = r#"PF\.obj\.config\.auth_token = "([a-f0-9]{40})";"#;
        let re = Regex::new(pattern).unwrap();

        let headers = resp.headers().clone();
        let status = resp.status();

        debug!(
            "got a response, status = {}, headers = {:?}",
            status, headers
        );

        if status != StatusCode::OK {
            error!("could not get the auth data: status = {}", status);
            return Err(Error::Status(status));
        }

        debug!("got the response of auth data");

        let html = resp.text().await?;

        if let Some(captures) = re.captures(&html) {
            let auth_token = match captures.get(1) {
                Some(t) => t.as_str(),
                None => return Err(Error::Other("解析 auth_token 失败".to_owned())),
            };

            debug!("got a new auth data: {}", auth_token);

            return Ok(Some((auth_token.to_owned(), headers)));
        }

        error!("couldn't get auth data, there is no auth token in response");

        Ok(None)
    }

    async fn update_auth_token(&mut self) -> Result<()> {
        trace!("updating auth_token");

        let data = self.get_auth_data(false).await?;

        match data {
            None => Err(Error::Other("自动更新认证信息失败".to_owned())),
            Some((s, _)) => {
                self.token = Some(s.clone());

                // 代码执行到此处时配置文件一定存在，不需要进行 None 判断。
                let mut config = CONFIG.read().await.clone().unwrap();
                let auth_config = ManagerAuthConfigKind::Chevereto {
                    username: self.username.clone(),
                    password: self.password.clone(),
                    timeout: Some(self.inner.timeout.as_secs().try_into().unwrap()),
                    extra: Some(HashMap::from([
                        ("cookie".into(), self.cookie.clone().unwrap()),
                        ("token".into(), self.token.clone().unwrap()),
                    ])),
                };
                config.insert_auth_config(self.code.clone(), auth_config);

                write_config(&config)?;
                info!("config file have been updated");

                Ok(())
            }
        }
    }

    async fn parse_auth_token_and_cookie(&self) -> Result<(String, String)> {
        trace!("parsing auth token and cookie");

        let data = self.get_auth_data(true).await?;
        match data {
            None => Err(Error::Other("解析 auth_token 和 cookie 失败".to_owned())),
            Some((s, headers)) => {
                let cookies = match headers.get("Set-Cookie") {
                    Some(c) => c,
                    None => return Err(Error::Other("解析 cookies 失败".to_owned())),
                };

                let cookies_str = cookies.to_str().map_err(|e| Error::Other(e.to_string()))?;
                let cookie = cookies_str.split(';').collect::<Vec<&str>>()[0];

                info!("got auth data, token = {}, cookie = {}", s, cookie);

                Ok((s, cookie.to_owned()))
            }
        }
    }

    async fn login(&mut self) -> Result<Extra> {
        trace!("log in {}", self.code.name());

        let (auth_token, cookie) = self.parse_auth_token_and_cookie().await?;

        let mut headers = self.header();
        headers.insert("Cookie", cookie.parse().unwrap());

        let url = self.inner.url("login");

        let params = [
            ("login-subject", &self.username),
            ("password", &self.password),
            ("auth_token", &auth_token),
        ];

        debug!("request: data = {:?}, headers = {:?}", params, headers);

        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let resp = client
            .post(url)
            .form(&params)
            .headers(headers)
            .send()
            .await?;

        let status = resp.status();
        let headers = resp.headers();

        debug!("got a resonse: status={}, headers={:?}", status, headers);

        if status != StatusCode::MOVED_PERMANENTLY {
            let html = resp.text().await?;

            let re = Regex::new(r#"PF\.fn\.growl\.expirable\("(.+?)"\)"#).unwrap();

            if let Some(captures) = re.captures(&html) {
                let error_detail = captures.get(1).unwrap().as_str();

                if error_detail == "错误的用户名或密码" {
                    return Err(Error::Auth);
                } else {
                    return Err(Error::Other(error_detail.to_owned()));
                }
            }

            return Err(Error::Status(status));
        }

        let cookies = match resp.headers().get("Set-Cookie") {
            Some(c) => c,
            None => return Err(Error::Other("cookies 为空".to_owned())),
        };
        let cookies_str = cookies.to_str().map_err(|e| Error::Other(e.to_string()))?;
        let keeplogin: &str = cookies_str.split(';').collect::<Vec<&str>>()[0];

        let cookie = format!("{}; {}", cookie, keeplogin);

        info!("log in - got cookies: {}", cookie);

        self.cookie = Some(cookie);
        self.token = Some(auth_token);

        Ok(HashMap::from([
            ("token".to_owned(), self.token.clone().unwrap()),
            ("cookie".to_owned(), self.cookie.clone().unwrap()),
        ]))
    }

    fn header(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        header.insert("Accept", "application/json".parse().unwrap());
        header.insert(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                .parse()
                .unwrap(),
        );

        if let Some(cookie) = &self.cookie {
            header.insert("Cookie", cookie.parse().unwrap());
        }

        header
    }

    #[async_recursion]
    async fn visit_page(
        &self,
        page: u8,
        seek: Option<String>,
        images: &mut Vec<ImageItem>,
    ) -> Result<()> {
        debug!("requesting ablum: page={}", page);

        let url = match seek {
            None => format!("{}/?page={}", self.inner.url(&self.username), page),
            Some(s) => format!(
                "{}/?page={}&seek={}",
                self.inner.url(&self.username),
                page,
                s
            ),
        };

        let resp = self.inner.get(&url, self.header()).await?;

        let status = resp.status();

        debug!("got a resonse: status={}", status);

        if status != StatusCode::OK {
            error!("got images list failed: status={}", status);
            return Err(Error::Status(status));
        }

        let text = resp.text().await?;
        let pattern = r#"data-object='(.+?)'"#;
        let re = Regex::new(pattern).unwrap();

        for (_, [s]) in re.captures_iter(&text).map(|c| c.extract()) {
            trace!("quoted image data: {}", s);
            let unquoted = unquote(s);
            debug!("unquoted image data: {}", unquoted);
            let image: CheveretoImage = serde_json::from_str(&unquoted)?;
            images.push(ImageItem {
                url: image.url,
                deleted_id: image.name,
                thumb: Some(image.thumb.url),
            });
        }

        // 当前页码大于 1 时不执行下面的代码
        if page > 1 {
            return Ok(());
        }

        // 每页 80 张图片，通过总数计算页数
        let image_count_re = Regex::new(r#"<b data-text="image-count">(\d+)</b>"#).unwrap();
        if let Some(captures) = image_count_re.captures(&text) {
            let image_count = captures.get(1).unwrap().as_str().parse::<u16>().unwrap();

            info!("got image count: {}", image_count);

            let pages = (image_count as f64 / 80.0).ceil() as u8;

            if pages == 1 {
                return Ok(());
            }

            let seek_re = Regex::new("&seek=(.+?)\"").unwrap();
            let seek = seek_re.captures(&text).unwrap().get(1).unwrap().as_str();

            for page in 2..(pages + 1) {
                self.visit_page(page, Some(seek.to_owned()), images).await?;
            }
        }

        Ok(())
    }

    async fn get_user_images(&self) -> Result<Vec<ImageItem>> {
        trace!("getting images");

        let mut images = vec![];

        self.visit_page(1, None, &mut images).await?;

        info!("got all images");

        Ok(images)
    }

    #[async_recursion]
    async fn delete_image_by_id(&mut self, id: &str, retry_counter: u8) -> Result<DeleteResponse> {
        trace!("deleting an image");

        let url = self.inner.url("json");

        let form = HashMap::from([
            ("auth_token", self.token.clone().unwrap()),
            ("action", "delete".to_string()),
            ("from", "list".to_string()),
            ("delete", "images".to_string()),
            ("multiple", "true".to_owned()),
            ("deleting[ids][]", id.to_string()),
        ]);

        let headers = self.header();
        debug!("request: form={:?}, headers={:?}", form, headers);

        let request = self.inner.request(Method::POST, &url, headers);
        let response = request.form(&form).send().await?;

        let status = response.status();

        debug!("got a response: status={}", status);

        if status != StatusCode::OK {
            let error_response = response.json::<CheveretoErrorResponse>().await?;
            debug!("deleted failed: {error_response:?}");

            let error: CheveretoError = error_response.error.into();

            match error {
                CheveretoError::InvalidContentOwnerRequest => {
                    error!("image is not exists: {}", id);
                    return Ok(DeleteResponse {
                        success: false,
                        error: Some(DeleteError::NotFound),
                    });
                }
                CheveretoError::Other(s) => {
                    error!("deleted failed: id={}, error={}", id, s);
                    return Ok(DeleteResponse {
                        success: false,
                        error: Some(DeleteError::Other(s)),
                    });
                }
                _ => {}
            }

            if retry_counter == MAX_RETRY_COUNT {
                return Ok(DeleteResponse {
                    success: false,
                    error: Some(DeleteError::Other(error.to_string())),
                });
            }

            info!("deletion failed due to an invalid auth token. Will update the auth token and retry: {}/{}", retry_counter, MAX_RETRY_COUNT);

            self.update_auth_token().await?;

            return self.delete_image_by_id(id, retry_counter + 1).await;
        }

        info!("successfully deleted the image: id={}", id);

        Ok(DeleteResponse {
            success: true,
            error: None,
        })
    }

    #[async_recursion]
    async fn upload(
        &mut self,
        window: Option<Window>,
        id: u32,
        image_path: &Path,
        retry_counter: u8,
    ) -> Result<CheveretoUploadResponse> {
        trace!("uploading: {:?}", image_path);

        if self.token.is_none() || self.cookie.is_none() {
            warn!("缺少 token 或 cookie，重新获取");
            self.login().await?;
        }

        let url = self.inner.url("json");

        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:85.0) Gecko/20100101 Firefox/85.0"
                .parse()
                .unwrap(),
        );
        headers.insert("Cookie", self.cookie.as_ref().unwrap().parse().unwrap());

        let timestamp = now()?.as_millis();
        let form = &[
            ("type", "file"),
            ("action", "upload"),
            ("timestamp", &timestamp.to_string()),
            ("auth_token", self.token.as_ref().unwrap()),
            ("nsfw", "0"),
        ];
        debug!("request: form={:?}, headers={:?}", form, headers);

        let response = self
            .inner
            .upload_multipart(
                window.clone(),
                id,
                &url,
                headers,
                image_path,
                &self.file_part_name,
                &self.file_kind,
                Some(form),
            )
            .await?;

        let status = response.status();
        if status != StatusCode::OK {
            let error_response = match response.json::<CheveretoErrorResponse>().await {
                Ok(e) => e,
                Err(e) => {
                    error!("反序列化时出错：{}", e);
                    return Err(Error::Reqeust(e));
                }
            };
            debug!("请求错误，响应体：{error_response:?}");

            let error: CheveretoError = error_response.error.into();

            match error {
                CheveretoError::AuthToken => {}
                _ => {
                    error!("uploaded failed: {}", error);
                    return Err(Error::Chevereto(error));
                }
            }

            if retry_counter == MAX_RETRY_COUNT {
                return Err(Error::Chevereto(error));
            }

            info!("upload failed due to an invalid auth token. Will update the auth token and retry: {}/{}", retry_counter, MAX_RETRY_COUNT);

            self.update_auth_token().await?;

            return self.upload(window, id, image_path, retry_counter + 1).await;
        }

        match response.json::<CheveretoUploadResponse>().await {
            Ok(r) => {
                info!("successfully uploaded the image: {:?}", image_path);
                Ok(r)
            }
            Err(e) => {
                error!("反序例化时出错：{}", e);
                Err(Error::Reqeust(e))
            }
        }
    }
}
