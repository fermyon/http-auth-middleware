use crate::sdk::http::ResponseBuilder;
use crate::wasi::http::types::{ErrorCode, Response};

/// `login` returns the login page.
pub async fn login() -> Result<Response, ErrorCode> {
    const LOGIN_HTML: &[u8] = include_bytes!("../../login.html"); // TODO: this shouldn't be included statically.

    ResponseBuilder::new()
        .with_status_code(200)
        .with_header("content-type", "text/html")
        .inspect_err(|_| eprintln!("error setting content-type header"))?
        .binary(LOGIN_HTML)
}
