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
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(structural)]
    pub type DeerToolboxPlugin;
    #[wasm_bindgen(structural)]
    pub type TFile;

    #[wasm_bindgen(method)]
    pub fn displayError(this: &DeerToolboxPlugin, error: String, file: Option<TFile>);

    #[wasm_bindgen(method, structural, js_class = "DeerToolboxPlugin")]
    pub fn mediaPath(this: &DeerToolboxPlugin) -> String;

    #[wasm_bindgen(method, structural, js_class = "DeerToolboxPlugin")]
    pub fn getActiveFileContent(this: &DeerToolboxPlugin) -> JsValue;

    // #[wasm_bindgen(method, getter)]
    // pub fn mediaPath(this: &DeerToolboxPlugin) -> String;

    #[wasm_bindgen(method, structural, js_class = "DeerToolboxPlugin")]
    pub fn saveBinaryResource(
        this: &DeerToolboxPlugin,
        fileName: &str,
        fileData: JsValue,
    ) -> JsValue;

    #[wasm_bindgen(method, structural, js_class = "DeerToolboxPlugin")]
    pub fn saveFileContent(this: &DeerToolboxPlugin, content: &str) -> JsValue;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
#[wasm_bindgen]
pub fn test_log(plugin: &DeerToolboxPlugin) {
    console_log!("I'm in...")
}

#[wasm_bindgen]
pub fn process_web_image(plugin: &DeerToolboxPlugin) {
    plugin.displayError("123".to_string(), None);
}

#[wasm_bindgen]
pub async fn process_web_content(plugin: &str) -> Result<JsValue, JsValue> {
    // plugin.displayError("123".to_string(), None);
    console_log!("{plugin}");
    Ok(JsValue::from_str("456"))
    // future_to_promise(process_web_image_from_content(plugin))
}

// #[wasm_bindgen]
// pub fn process_web_content2(plugin: &DeerToolboxPlugin) -> Promise {
//     plugin.displayError("123".to_string(), None);
//     future_to_promise(process_web_image_from_content2(plugin))
// }

// pub async fn process_web_image_from_content2(
//     plugin: &DeerToolboxPlugin,
// ) -> Result<JsValue, JsValue> {
//     plugin.displayError("4455".to_string(), None);
//     Ok(JsValue::from_str("4455"))
// }

#[wasm_bindgen]
pub async fn process_web_image_from_content(
    plugin: &DeerToolboxPlugin,
) -> Result<JsValue, JsValue> {
    console_log!("process_web_image_from_content");
    let mut result_promise = plugin.getActiveFileContent();
    let mut promise = js_sys::Promise::from(result_promise);
    let mut future = JsFuture::from(promise);
    let result: Result<JsValue, JsValue> = future.await;
    if let Err(e) = result {
        return Err(e);
    }
    let content_js_value = result.unwrap();
    let content = content_js_value.as_string().unwrap();

    let md_image_reg = r"!\[(.*?)\]\((.+?)\)";
    // match all image url from markdown content
    let re = regex::Regex::new(md_image_reg).unwrap();
    let mut url_list: Vec<&str> = Vec::new();
    for cap in re.captures_iter(&content) {
        let url = cap.get(2).unwrap().as_str();
        url_list.push(url);
    }
    console_log!("url_list: {:?}", url_list);
    let url_uuid_map = handle_imge_processing(plugin, url_list).await?;
    let replaced_content = re.replace_all(&content, |caps: &regex::Captures| {
        let anchor = caps.get(1).unwrap().as_str();
        let url = caps.get(2).unwrap().as_str();
        let uuid = url_uuid_map.get(url).unwrap();
        console_log!("![{}]({})", anchor, uuid);
        format!("![{}]({})", anchor, uuid)
    });
    console_log!("before replaced_content");
    console_log!("{replaced_content}");

    result_promise = plugin.saveFileContent(replaced_content.as_ref());
    promise = js_sys::Promise::from(result_promise);
    future = JsFuture::from(promise);
    let result: Result<JsValue, JsValue> = future.await;
    if let Err(e) = result {
        return Err(e);
    }

    // let Ok(content_js_value) = result;
    console_log!("after replaced_content");
    Ok(JsValue::UNDEFINED)
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
    console_log!("fetch_image: {}", url);
    let req = Request::new_with_str(url)?;
    // req.headers().set("Content-Type", "image/png")?;
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
    let result_promise = plugin.saveBinaryResource(file_name.as_str(), JsValue::from(file_data));
    let result: Result<JsValue, JsValue> =
        JsFuture::from(js_sys::Promise::from(result_promise)).await;
    if let Err(e) = result {
        return Err(e);
    }
    Ok(file_name)
}
