use crate::wasi::http::types::{Body, ErrorCode, Headers, Request, Response, Scheme};
use crate::wit_stream;
use cookie::Cookie;
use futures::SinkExt;
use http::header::COOKIE;

/// `authenticate` validates the access token required in the incoming request by making an
/// outgoing request to github. If the token is valid, the request is passed through to the
/// imported endpoint.
pub async fn authenticate(request: Request) -> Result<Response, ErrorCode> {
    let token = match get_access_token(&request) {
        Some(token) => token,
        None => {
            eprintln!("no access token found in incoming request");

            return Ok(unauthorized());
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

                Ok(unauthorized())
            }
        }
        Err(error) => {
            eprintln!("error authenticating with github: {error}");
            let response = Response::new(Headers::new(), None);
            response.set_status_code(500).unwrap();
            Ok(response)
        }
    }
}

fn unauthorized() -> Response {
    let headers = Headers::new();
    headers
        .set("Content-Type", &[b"text/html".to_vec()])
        .unwrap();

    let (mut writer, reader) = wit_stream::new();
    let (body, _err_fut) = Body::new(reader);

    let response = Response::new(headers, Some(body));
    response.set_status_code(403).unwrap();

    wit_bindgen_rt::async_support::spawn(async move {
        writer
            .send(b"Unauthorized, <a href=\"/login\">login</a>".into())
            .await
            .unwrap();
    });

    response
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
