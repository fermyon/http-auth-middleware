use spin_sdk::http::{Headers, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;
use url::Url;

mod api;

// TODO: Allow configurable redirect URL
#[http_component]
async fn middleware(request: IncomingRequest, output: ResponseOutparam) {
    let url = match get_url(&request) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("error parsing URL: {e}");
            let response = OutgoingResponse::new(Headers::new());
            response.set_status_code(500).unwrap();
            output.set(response);
            return;
        }
    };

    match url.path() {
        "/login/authorize" => api::authorize(output).await,
        "/login/callback" => api::callback(url, output).await,
        "/login" => api::login(output).await,
        _ => api::authenticate(request, output).await,
    }
}

fn get_url(request: &IncomingRequest) -> anyhow::Result<Url> {
    let authority = request
        .authority()
        .ok_or(anyhow::anyhow!("missing host header"))?;

    let path = request.path_with_query().unwrap_or_default();
    let full = format!("http://{}{}", authority, path);
    Ok(Url::parse(&full)?)
}

wit_bindgen::generate!({
    runtime_path: "::spin_sdk::wit_bindgen::rt",
    world: "wasi-http-import",
    path: "wits",
    with: {
        "wasi:http/types@0.2.0": spin_sdk::wit::wasi::http::types,
        "wasi:io/error@0.2.0": spin_executor::bindings::wasi::io::error,
        "wasi:io/streams@0.2.0": spin_executor::bindings::wasi::io::streams,
        "wasi:io/poll@0.2.0": spin_executor::bindings::wasi::io::poll,
    }
});
