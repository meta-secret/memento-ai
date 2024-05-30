use axum::{
    body::Body,
    http::{Request, header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_HEADERS}},
    middleware::Next,
    response::IntoResponse,
};
use std::convert::Infallible;
use tracing::info;

pub(crate) async fn cors_middleware(req: Request<Body>, next: Next) -> Result<impl IntoResponse, Infallible> {
    info!("cors_middleware");
    let mut response = next.run(req).await;

    response.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    response.headers_mut().insert(ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, POST, OPTIONS"));
    response.headers_mut().insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("Content-Type, Authorization"));

    info!("cors_middleware response {:?}", response);
    Ok(response)
}
