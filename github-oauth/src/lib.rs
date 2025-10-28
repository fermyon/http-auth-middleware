use spin_sdk::http_wasip3::{Request, IntoResponse};

mod api;
mod response;

wit_bindgen::generate!({
    path: "../wit",
    world: "double:http/middleware",
    async: true,
    with: {
        "wasi:http/types@0.3.0-rc-2025-09-16": spin_sdk::http_wasip3::wasip3::http::types,
    },
    generate_all,
});

#[spin_sdk::http_wasip3::http_service]
async fn handle(request: Request) -> impl IntoResponse {
    match request.uri().path() {
        "/login/authorize" => api::authorize().await.into_response(),
        "/login/callback" => api::callback(request.uri()).await.into_response(),
        "/login" => api::login().await.into_response(),
        _ => api::authenticate(request).await.into_response(),
    }
}
