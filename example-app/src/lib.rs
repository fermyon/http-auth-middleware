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

use wasi::http::types::{Body, ErrorCode, Headers, Request, Response};
use wit_bindgen_rt::async_support::futures::SinkExt;

struct ExampleApp;

impl exports::wasi::http::handler::Guest for ExampleApp {
    async fn handle(_request: Request) -> Result<Response, ErrorCode> {
        let (mut writer, reader) = wit_stream::new();
        let (body, _efut) = Body::new(reader);

        wit_bindgen_rt::async_support::spawn(async move {
            writer
                .send("Business logic executed!\n".into())
                .await
                .unwrap();
        });

        let headers = Headers::new();
        headers.set("content-type", &["text/plain".into()]).unwrap();

        Ok(Response::new(headers, Some(body)))
    }
}

export!(ExampleApp);
