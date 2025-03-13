use spin_sdk::http_wasip3::{FullBody, IntoResponse};

/// `login` returns the login page.
pub async fn login() -> impl IntoResponse {
    const LOGIN_HTML: &[u8] = include_bytes!("../../login.html"); // TODO: this shouldn't be included statically.

    http::Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(FullBody::new(LOGIN_HTML))
}
