use spin_sdk::http::{IncomingRequest, ResponseOutparam};
use spin_sdk::http_component;

wit_bindgen::generate!({ 
    runtime_path: "::spin_sdk::wit_bindgen::rt",
    world: "wasi-http-import",
    path: "wit",
    with: {
        "wasi:http/types@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::http::types,
        "wasi:io/streams@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::io::streams,
        "wasi:io/poll@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::io::poll,
    }
});

#[http_component]
async fn auth(req: IncomingRequest, out: ResponseOutparam) {
    // TODO: do actual auth with github
    println!("AUTH");
    wasi::http::incoming_handler::handle(req, out.into_inner());
}
