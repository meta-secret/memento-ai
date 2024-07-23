use thiserror::Error;
use wasm_bindgen::JsValue;
use std::fmt;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("js error: {0}")]
    JsError(JsValue),
}

impl From<MyError> for JsValue {
    fn from(error: MyError) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}
