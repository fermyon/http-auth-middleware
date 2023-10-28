use cookie::Cookie;
use http::header::COOKIE;
use spin_sdk::http::{send, Headers, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

#[http_component]
async fn middleware(request: IncomingRequest, output: ResponseOutparam) {
    match get_token(&request) {
        Some(token) => authenticate(request, output, &token).await,
        None => {
            let response = OutgoingResponse::new(403, &Headers::new(&[]));
            output.set(response);
        }
    }
}

fn get_token(req: &IncomingRequest) -> Option<String> {
    let cookies: Vec<Vec<u8>> = req.headers().get(COOKIE.as_str());
    for encoded in cookies {
        if let Ok(cookie) = Cookie::parse(String::from_utf8_lossy(&encoded)) {
            const OAUTH_TOKEN: &str = "token";
            if matches!(cookie.name(), OAUTH_TOKEN) {
                return Some(cookie.value().to_string());
            }
        }
    }
    None
}

async fn authenticate(request: IncomingRequest, output: ResponseOutparam, token: &str) {
    let result = send::<_, http::Response<()>>(
        http::Request::builder()
            .method("GET")
            .uri("https://api.github.com/octocat")
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "Spin Middleware")
            .body(())
            .unwrap(),
    )
    .await;

    match result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                wasi::http::incoming_handler::handle(request, output.into_inner());
            } else {
                eprintln!("unauthenticated");
                output.set(OutgoingResponse::new(status.as_u16(), &Headers::new(&[])));
            }
        }
        Err(error) => {
            eprintln!("error authenticating with github: {error}");
            output.set(OutgoingResponse::new(500, &Headers::new(&[])));
        }
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
