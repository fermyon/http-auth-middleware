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
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
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
    let mut host_header = request
        .headers()
        .get(http::header::HOST.as_str())
        .into_iter();
    let header = &host_header
        .next()
        .ok_or(anyhow::anyhow!("missing host header"))?;
    let host = String::from_utf8_lossy(header);
    let path = request.path_with_query().unwrap_or_default();
    let full = format!("http://{}{}", host, path);
    Ok(Url::parse(&full)?)
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
