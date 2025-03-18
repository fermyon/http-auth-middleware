use crate::sdk::http::ResponseBuilder;
use crate::wasi::http::types::{ErrorCode, Headers, Request, Response, Scheme};
use cookie::Cookie;
use http::header::COOKIE;

/// `authenticate` validates the access token required in the incoming request by making an
/// outgoing request to github. If the token is valid, the request is passed through to the
/// imported endpoint.
pub async fn authenticate(request: Request) -> Result<Response, ErrorCode> {
    let token = match get_access_token(&request) {
        Some(token) => token,
        None => {
            eprintln!("no access token found in incoming request");

            return unauthorized();
        }
    };

    let api_headers = Headers::new();
    api_headers
        .set("Authorization", &[format!("Bearer {token}").into()])
        .unwrap();
    api_headers
        .set("User-Agent", &["Spin Middleware".into()])
        .unwrap();
    let api_request = Request::new(api_headers, None, None);
    api_request.set_scheme(Some(&Scheme::Https)).unwrap();
    api_request.set_authority(Some("api.github.com")).unwrap();
    api_request.set_path_with_query(Some("/user")).unwrap();

    let api_response = crate::wasi::http::handler::handle(api_request).await;

    match api_response {
        Ok(response) => {
            let status = response.status_code();
            if status / 100 == 2 {
                eprintln!("authenticated");
                crate::double::http::chain_http::handle(request).await
            } else {
                eprintln!("unauthenticated");
                unauthorized()
            }
        }
        Err(error) => {
            eprintln!("error authenticating with github: {error}");
            ResponseBuilder::new().with_status_code(500).empty()
        }
    }
}

fn unauthorized() -> Result<Response, ErrorCode> {
    ResponseBuilder::new()
        .with_status_code(403)
        .with_header("Content-Type", "text/html")?
        .text("Unauthorized, <a href=\"/login\">login</a>")
}

fn get_access_token(request: &Request) -> Option<String> {
    let cookies: Vec<Vec<u8>> = request.headers().get(COOKIE.as_ref());
    for encoded in cookies {
        let parsed = Cookie::split_parse(String::from_utf8_lossy(&encoded));
        for cookie in parsed.flatten() {
            const OAUTH_TOKEN: &str = "access-token";
            if matches!(cookie.name(), OAUTH_TOKEN) {
                return Some(cookie.value().to_string());
            }
        }
    }
    None
}
