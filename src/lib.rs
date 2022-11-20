mod utils;

use file_format::FileFormat;
use js_sys::Uint8Array;
use md5_rs::Context;
use regex;
use std::collections::HashMap;
use uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    pub type DeerToolboxPlugin;
    pub type TFile;

    #[wasm_bindgen(structural, method)]
    pub fn displayError(this: &DeerToolboxPlugin, error: String, file: Option<TFile>);

    #[wasm_bindgen(catch, structural, method)]
    pub async fn getActiveFileContent(this: &DeerToolboxPlugin) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn mediaPath(this: &DeerToolboxPlugin) -> String;

    #[wasm_bindgen(catch, structural, method)]
    pub async fn saveBinaryResource(
        this: &DeerToolboxPlugin,
        fileName: &str,
        fileData: JsValue,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, structural, method)]
    pub async fn saveFileContent(this: &DeerToolboxPlugin, content: String) -> Result<(), JsValue>;
}

#[wasm_bindgen]
pub async fn process_web_image_from_content(plugin: &DeerToolboxPlugin) -> Result<(), JsValue> {
    let content_js_value = plugin.getActiveFileContent().await?;
    let content = content_js_value.as_string().unwrap();
    let md_image_reg = r"!\[(.*)\]\((.*)\)";
    // match all image url from markdown content
    let re = regex::Regex::new(md_image_reg).unwrap();
    let mut url_list: Vec<&str> = Vec::new();
    for cap in re.captures_iter(&content) {
        let url = cap.get(2).unwrap().as_str();
        url_list.push(url);
    }
    let url_uuid_map = handle_imge_processing(plugin, url_list).await?;
    let replaced_content = re.replace_all(md_image_reg, |caps: &regex::Captures| {
        let anchor = caps.get(1).unwrap().as_str();
        let url = caps.get(2).unwrap().as_str();
        let uuid = url_uuid_map.get(url).unwrap();
        format!("![{}]({})", anchor, uuid)
    });

    plugin.saveFileContent(replaced_content.to_string()).await?;
    Ok(())
}

async fn handle_imge_processing(
    plugin: &DeerToolboxPlugin,
    url_list: Vec<&str>,
) -> Result<HashMap<String, String>, JsValue> {
    let mut url_uuid_map: HashMap<String, String> = HashMap::with_capacity(url_list.len());
    let mut md5_uuid_map: HashMap<String, String> = HashMap::with_capacity(url_list.len());
    for url in url_list {
        if url_uuid_map.contains_key(url) {
            continue;
        }
        let image_buffer = fetch_image(url).await?;
        let image_md5 = calculate_md5(&image_buffer);
        if md5_uuid_map.contains_key(image_md5.as_str()) {
            url_uuid_map.insert(
                url.to_string().clone(),
                md5_uuid_map.get(image_md5.as_str()).unwrap().clone(),
            );
        } else {
            let file_name = save_resource(plugin, image_buffer).await?;
            url_uuid_map.insert(url.to_string().clone(), file_name.clone());
            md5_uuid_map.insert(image_md5.clone(), file_name);
        }
    }
    Ok(url_uuid_map)
}

async fn fetch_image(url: &str) -> Result<Vec<u8>, JsValue> {
    let req = Request::new_with_str(url)?;
    req.headers().set("Content-Type", "image/png")?;
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
    let bytes = Uint8Array::new(&array_buffer).to_vec();
    Ok(bytes)
}

fn calculate_md5(bytes: &[u8]) -> String {
    let mut context = Context::new();
    context.read(bytes);
    let digest = context.finish();
    let hash = digest
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    return hash;
}

async fn save_resource(plugin: &DeerToolboxPlugin, buf: Vec<u8>) -> Result<String, JsValue> {
    let format = FileFormat::from_bytes(&buf);
    let ext = format.extension();
    let image_uuid = uuid::Uuid::new_v4().to_string();
    let media_root_path = plugin.mediaPath();
    let file_name = format!("{}/{}.{}", media_root_path, image_uuid, ext);
    let file_data = js_sys::Uint8Array::from(&buf[..]);
    plugin
        .saveBinaryResource(file_name.as_str(), JsValue::from(file_data))
        .await?;
    Ok(file_name)
}
