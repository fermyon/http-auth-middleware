use url::Url;

mod api;

wit_bindgen::generate!({
    path: "../wit",
    world: "double:http/proxy",
    async: {
        exports: [
            "wasi:http/handler@0.3.0-draft#handle",
        ],
        imports: [
            "wasi:http/handler@0.3.0-draft#handle",
            "double:http/chain-http#handle",
        ]
    },
    generate_all,
});

use wasi::http::types::{ErrorCode, Headers, Request, Response, Scheme};

struct Middleware;

impl exports::wasi::http::handler::Guest for Middleware {
    async fn handle(request: Request) -> Result<Response, ErrorCode> {
        let url = match get_url(&request) {
            Ok(url) => url,
            Err(e) => {
                eprintln!("error parsing URL: {e}");
                let response = Response::new(Headers::new(), None);
                response.set_status_code(500).unwrap();
                return Ok(response);
            }
        };

        match url.path() {
            "/login/authorize" => api::authorize().await,
            "/login/callback" => api::callback(url).await,
            "/login" => api::login().await,
            _ => api::authenticate(request).await,
        }
    }
}

export!(Middleware);

fn get_url(request: &Request) -> anyhow::Result<Url> {
    let authority = request
        .authority()
        .ok_or(anyhow::anyhow!("missing host header"))?;

    let path = request.path_with_query().unwrap_or_default();

    let scheme = match request.scheme() {
        None => "http".to_owned(),
        Some(Scheme::Http) => "http".to_owned(),
        Some(Scheme::Https) => "https".to_owned(),
        Some(Scheme::Other(s)) => s,
    };

    let full = format!("{scheme}://{authority}{path}");
    Ok(Url::parse(&full)?)
}
