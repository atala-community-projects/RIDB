
use serde_wasm_bindgen::from_value;
use wasm_bindgen::{ JsValue};

pub fn extract_property<T>(js_value: &JsValue, key: &str) -> Result<T, JsValue>
    where
        T: for<'de> serde::Deserialize<'de>,
{
    let prop: JsValue = if js_value.is_object() {
        let property = js_sys::Reflect::get(js_value, &JsValue::from(key)).expect("Error getting property");
        property
    } else {
        JsValue::from(js_value)
    };
    from_value(prop).map_err(|err| JsValue::from(err.to_string()))
}

