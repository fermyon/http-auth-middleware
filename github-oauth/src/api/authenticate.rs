use crate::response;

use spin_sdk::http_wasip3::{send, EmptyBody, Request, IntoRequest, IntoResponse};

use cookie::Cookie;
use http::header::COOKIE;

/// `authenticate` validates the access token required in the incoming request by making an
/// outgoing request to github. If the token is valid, the request is passed through to the
/// imported endpoint.
pub async fn authenticate(mut request: Request) -> impl IntoResponse {
    let token = match get_access_token(&request) {
        Some(token) => token,
        None => {
            eprintln!("no access token found in incoming request");

            return unauthorized().into_response();
        }
    };

    let api_request = http::Request::builder()
        .uri("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", "Spin Middleware")
        .body(EmptyBody::new())
        .unwrap();

    let api_response = send(api_request).await;

    match api_response {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                eprintln!("authenticated");
                // mmm, forbidden header
                request.headers_mut().remove("connection");
                request.headers_mut().remove("host");
                crate::double::http::chain_http::handle(request.into_request()?).await.into_response()
            } else {
                eprintln!("unauthenticated");
                unauthorized().into_response()
            }
        }
        Err(error) => {
            eprintln!("error authenticating with github: {error}");
            response::internal_server_error().into_response()
        }
    }
}

fn unauthorized() -> impl IntoResponse {
    http::Response::builder()
        .status(403)
        .header("Content-Type", "text/html")
        .body("Unauthorized, <a href=\"/login\">login</a>".to_owned())
}

fn get_access_token(request: &Request) -> Option<String> {
    const OAUTH_TOKEN: &str = "access-token";

    let cookies = request.headers().get(&COOKIE)?.to_str().ok()?;

    let parsed = Cookie::split_parse(cookies);
    for cookie in parsed.flatten() {
        if cookie.name() == OAUTH_TOKEN {
            return Some(cookie.value().to_string());
        }
    }

    None
}
