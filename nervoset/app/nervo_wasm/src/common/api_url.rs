use crate::run_mode::ClientRunMode;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct ApiUrl {
    url: &'static str,
    port: u32,
    run_mode: ClientRunMode,
}

#[wasm_bindgen]
impl ApiUrl {
    pub fn get(port: u32, run_mode: ClientRunMode) -> Self {
        match run_mode {
            ClientRunMode::Local => ApiUrl::local(port),
            ClientRunMode::Dev => ApiUrl::dev(port),
            ClientRunMode::Prod => ApiUrl::prod(),
        }
    }

    pub fn local(port: u32) -> Self {
        ApiUrl {
            url: "http://localhost",
            port,
            run_mode: ClientRunMode::Local,
        }
    }
    pub fn dev(port: u32) -> Self {
        ApiUrl {
            url: "http://nervoset.metaelon.space",
            port,
            run_mode: ClientRunMode::Dev,
        }
    }

    pub fn prod() -> Self {
        ApiUrl {
            url: "https://prod.metaelon.space",
            port: 443,
            run_mode: ClientRunMode::Prod,
        }
    }
}

#[wasm_bindgen]
impl ApiUrl {
    pub fn get_url(&self) -> String {
        format!("{}:{}", self.url, self.port)
    }
}
