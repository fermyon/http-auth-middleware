use spin_sdk::http::{Headers, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;
use url::Url;

mod api;

// TODO: Allow configurable redirect URL

#[http_component]
async fn middleware(request: IncomingRequest, output: ResponseOutparam) {
    let url = match Url::parse(&request.uri()) {
        Ok(url) => url,
        Err(error) => {
            eprintln!("error parsing URL: {error}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            output.set(response);
            return;
        }
    };

    match url.path() {
        "/login/authorize" => api::authorize(request, output).await,
        "/login/callback" => api::callback(url, request, output).await,
        "/login" => api::login(request, output).await,
        _ => api::authenticate(request, output).await,
    }
}

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
