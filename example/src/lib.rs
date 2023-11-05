use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_component;

/// A simple HTTP component
#[http_component]
fn handle_http_handler(_req: Request) -> anyhow::Result<impl IntoResponse> {
    Ok(http::Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Business logic executed!")?)
}
