use spin_sdk::http::{IncomingRequest, ResponseOutparam};
use spin_sdk::http_component;

wit_bindgen::generate!({ 
    runtime_path: "::spin_sdk::wit_bindgen::rt",
    world: "wasi-http-import",
    path: "wit",
    with: {
        "wasi:http/types@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::http::types,
        "wasi:io/streams@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::io::streams,
        "wasi:io/poll@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::io,
    }
});


/// A simple Spin HTTP component.
#[http_component]
async fn auth(req: IncomingRequest, out: ResponseOutparam) {
    wasi::http::incoming_handler::handle(req, out);

    // println!("{:?}", req.headers);
    // Ok(http::Response::builder()
    //     .status(200)
    //     .header("content-type", "text/plain")
    //     .body("Hello, Fermyon")?)
    todo!("")
}
