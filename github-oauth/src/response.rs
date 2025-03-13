use spin_sdk::http_wasip3::{EmptyBody, IntoResponse};

/// 307 Temporary Redirect with no header other than Location
pub fn temporary_redirect(location: &str) -> impl IntoResponse {
    http::Response::builder()
        .status(307)
        .header("Location", location)
        .body(EmptyBody::new())
}

/// 403 Forbidden with no message body
pub fn forbidden() -> impl IntoResponse {
    http::Response::builder().status(403).body(EmptyBody::new())
}

/// 500 Internal Server Error with no message body
pub fn internal_server_error() -> impl IntoResponse {
    http::Response::builder().status(500).body(EmptyBody::new())
}
