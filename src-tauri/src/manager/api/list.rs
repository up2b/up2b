use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    manager::{api::SerdeValueParser, ImageItem},
    Up2bError, Up2bResult,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum ListRequestMethod {
    Get,
    Post { body: Map<String, Value> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List {
    pub(super) path: String,
    pub(super) controller: ListResponseController,
    pub(super) method: ListRequestMethod,
}

impl List {
    pub fn new(url: &str, controller: ListResponseController, method: ListRequestMethod) -> Self {
        Self {
            path: url.to_owned(),
            controller,
            method,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListResponseController {
    items_key: String,
    image_url_key: String,
    thumb_key: Option<String>,
    deleted_id_key: String,
}

impl ListResponseController {
    pub fn new<S: Into<String>, O: Into<Option<String>>>(
        items_key: S,
        image_url_key: S,
        deleted_id_key: S,
        thumb_key: O,
    ) -> Self {
        Self {
            items_key: items_key.into(),
            image_url_key: image_url_key.into(),
            deleted_id_key: deleted_id_key.into(),
            thumb_key: thumb_key.into(),
        }
    }

    pub(super) async fn parse(&self, response: Response) -> Up2bResult<Vec<ImageItem>> {
        let json: Value = response.json().await?;

        let items = match json.get_value_by_keys(&self.items_key) {
            Value::Array(arr) => arr,
            _ => return Err(Up2bError::Other("通过 items_key 无法获取列表".to_owned())),
        };

        println!("{:?}", items);

        let mut images = Vec::with_capacity(items.len());

        for item in items.iter() {
            let url = match item.get_value_by_keys(&self.image_url_key) {
                Value::String(s) => s,
                _ => return Err(Up2bError::Other("没有图片链接".to_owned())),
            };
            let deleted_id = match item.get_value_by_keys(&self.deleted_id_key) {
                Value::String(s) => s,
                _ => return Err(Up2bError::Other("没有删除 id".to_owned())),
            };

            let image_item = match &self.thumb_key {
                None => ImageItem {
                    url,
                    deleted_id,
                    thumb: None,
                },
                Some(k) => {
                    let thumb = match item.get_value_by_keys(k) {
                        Value::String(s) => Some(s),
                        _ => None,
                    };

                    debug!("thumb: key={} value={:?}", k, thumb);

                    ImageItem {
                        url,
                        deleted_id,
                        thumb,
                    }
                }
            };

            images.push(image_item);
        }

        debug!("图片列表：{:?}", images);

        Ok(images)
    }
}
