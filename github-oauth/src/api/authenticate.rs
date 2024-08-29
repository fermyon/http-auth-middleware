use cookie::Cookie;
use futures::SinkExt;
use http::header::COOKIE;
use spin_sdk::http::{send, Headers, IncomingRequest, OutgoingResponse, ResponseOutparam};

/// `authenticate` validates the access token required in the incoming request by making an
/// outgoing request to github. If the token is valid, the request is passed through to the
/// imported endpoint.
pub async fn authenticate(request: IncomingRequest, output: ResponseOutparam) {
    let token = match get_access_token(&request) {
        Some(token) => token,
        None => {
            eprintln!("no access token found in incoming request");

            let headers = Headers::new();
            headers
                .set(&"Content-Type".to_string(), &[b"text/html".to_vec()])
                .unwrap();

            let response = OutgoingResponse::new(headers);
            response.set_status_code(403).unwrap();

            let mut body = response.take_body();
            output.set(response);

            if let Err(error) = body
                .send(b"Unauthorized, <a href=\"/login\">login</a>".to_vec())
                .await
            {
                eprintln!("error send login page: {error}");
            }

            return;
        }
    };

    let result = send::<_, http::Response<()>>(
        http::Request::builder()
            .method("GET")
            .uri("https://api.github.com/user")
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
                eprintln!("authenticated");
                crate::wasi::http::incoming_handler::handle(request, output.into_inner());
            } else {
                eprintln!("unauthenticated");

                let headers = Headers::new();
                headers
                    .set(&"Content-Type".to_string(), &[b"text/html".to_vec()])
                    .unwrap();

                let response = OutgoingResponse::new(headers);
                response.set_status_code(status.as_u16()).unwrap();

                let mut body = response.take_body();
                output.set(response);

                if let Err(error) = body
                    .send(b"Unauthorized, <a href=\"/login\">login</a>".to_vec())
                    .await
                {
                    eprintln!("error sending page: {error}");
                }
            }
        }
        Err(error) => {
            eprintln!("error authenticating with github: {error}");
            let response = OutgoingResponse::new(Headers::new());
            response.set_status_code(500).unwrap();
            output.set(response);
        }
    }
}

fn get_access_token(request: &IncomingRequest) -> Option<String> {
    let cookies: Vec<Vec<u8>> = request.headers().get(&COOKIE.to_string());
    for encoded in cookies {
        let parsed = Cookie::split_parse(String::from_utf8_lossy(&encoded));
        for cookie in parsed {
            if let Ok(cookie) = cookie {
                const OAUTH_TOKEN: &str = "access-token";
                if matches!(cookie.name(), OAUTH_TOKEN) {
                    return Some(cookie.value().to_string());
                }
            }
        }
    }
    None
}
