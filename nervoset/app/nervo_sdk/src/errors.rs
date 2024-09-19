use thiserror::Error;
use wasm_bindgen::prelude::*;

pub type NervoWebResult<T> = Result<T, NervoSdkError>;

#[derive(Error, Debug)]
pub enum NervoSdkError {
    #[error("Unknown run mode: {0}")]
    UnknownRunModeError(String),

    #[error("Unknown App Type: {0}")]
    UnknownAppTypeError(String),
}

impl From<NervoSdkError> for JsValue {
    fn from(error: NervoSdkError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}
