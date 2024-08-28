use thiserror::Error;
use wasm_bindgen::prelude::*;

pub type NervoWebResult<T> = Result<T, NervoWebError>;

#[derive(Error, Debug)]
pub enum NervoWebError {
    #[error("Unknown run mode: {0}")]
    UnknownRunModeError(String),
}

impl From<NervoWebError> for JsValue {
    fn from(error: NervoWebError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}