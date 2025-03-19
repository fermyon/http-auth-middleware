wit_bindgen::generate!({
    path: "../wit/wasi-http/wit-0.3.0-draft",
    world: "wasi:http/proxy",
    async: {
        exports: [
            "wasi:http/handler@0.3.0-draft#handle",
        ],
    },
    generate_all,
});

mod sdk;

use wasi::http::types::{ErrorCode, Request, Response};

struct ExampleApp;

impl exports::wasi::http::handler::Guest for ExampleApp {
    async fn handle(_request: Request) -> Result<Response, ErrorCode> {
        sdk::http::ResponseBuilder::new()
            .with_status_code(200)
            .with_header("content-type", "text/plain")?
            .text("Business logic executed!\n")
    }
}

export!(ExampleApp);
