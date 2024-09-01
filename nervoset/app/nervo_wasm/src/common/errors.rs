use error_stack::{Report, ResultExt};
use thiserror::Error;
use wasm_bindgen::JsValue;

pub type NervoWebAppResult<T> = Result<T, NervoWebAppError>;

#[derive(Error, Debug)]
pub enum NervoWebAppError {
    #[error("Dangerous error: {0}.")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Dangerous error")]
    SomeError,
}

impl From<NervoWebAppError> for JsValue {
    fn from(error: NervoWebAppError) -> Self {
        let err_result: Result<(), Report<NervoWebAppError>> = Err(error)
            .attach_printable_lazy(|| String::from("Nervo error:"));
        serde_wasm_bindgen::to_value(&err_result).unwrap()
    }
}