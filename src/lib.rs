use cookie::{Cookie, SameSite};
use futures::SinkExt;
use http::header::COOKIE;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use spin_sdk::http::{
    send, Headers, IncomingRequest, OutgoingResponse, ResponseOutparam, SendError,
};
use spin_sdk::http_component;
use url::Url;

// TODO: SPIN_HTTP_AUTH_MIDDLEWARE_ERROR_URL
// TODO: SPIN_HTTP_AUTH_MIDDLEWARE_SUCCESS_URL
// TODO: SPIN_HTTP_AUTH_MIDDLEWARE_SCOPES
// TODO: SPIN_HTTP_AUTH_MIDDLEWARE_CLIENT_ID
// TODO: SPIN_HTTP_AUTH_MIDDLEWARE_CLIENT_SECRET

#[http_component]
async fn middleware(request: IncomingRequest, output: ResponseOutparam) {
    let url = match Url::parse(&request.uri()) {
        Ok(url) => url,
        Err(error) => {
            eprintln!("error parsing URL: {error}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            output.set(response);
            return;
        }
    };

    match url.path() {
        "/login/authorize" => authorize(request, output).await,
        "/login/callback" => callback(url, request, output).await,
        "/login" => login(request, output).await,
        _ => authenticate(request, output).await,
    }
}

struct OAuth2 {
    client_secret: ClientSecret,
    client_id: ClientId,
    auth_url: AuthUrl,
    token_url: TokenUrl,
}

impl OAuth2 {
    fn try_init() -> anyhow::Result<Self> {
        let client_secret = ClientSecret::new(std::env::var("CLIENT_SECRET")?);
        let client_id = ClientId::new(std::env::var("CLIENT_ID")?);
        let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?;
        let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?;

        Ok(OAuth2 {
            client_secret,
            client_id,
            token_url,
            auth_url,
        })
    }

    fn into_client(self) -> BasicClient {
        BasicClient::new(
            self.client_id,
            Some(self.client_secret),
            self.auth_url,
            Some(self.token_url),
        )
    }
}

async fn login(request: IncomingRequest, output: ResponseOutparam) {
    const LOGIN_HTML: &[u8] = include_bytes!("../login.html");

    let response = OutgoingResponse::new(
        200,
        &Headers::new(&[("content-type".to_string(), b"text/html".to_vec())]),
    );

    let mut body = response.take_body();
    output.set(response);

    if let Err(error) = body.send(LOGIN_HTML.to_vec()).await {
        eprintln!("error send login page: {error}");
    }
}

async fn authorize(_request: IncomingRequest, output: ResponseOutparam) {
    let client = match OAuth2::try_init() {
        Ok(config) => {
            let redirect_url = RedirectUrl::new("http://127.0.0.1:3000/login/callback".to_string())
                .expect("Invalid redirect URL");
            config
                .into_client()
                .set_auth_type(oauth2::AuthType::RequestBody)
                .set_redirect_uri(redirect_url)
        }
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            output.set(response);
            return;
        }
    };

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the user's public repos and email.
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    let location = authorize_url.to_string().as_bytes().to_vec();
    let headers = Headers::new(&[("Location".to_string(), location)]);
    let response = OutgoingResponse::new(301, &headers);
    output.set(response);
}

async fn callback(url: Url, request: IncomingRequest, output: ResponseOutparam) {
    fn get_query_param(url: &Url, param: &str) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| key == param)
            .map(|(_, value)| value.into_owned())
    }

    let client = match OAuth2::try_init() {
        Ok(config) => {
            let redirect_url = RedirectUrl::new("http://127.0.0.1:3000/login/callback".to_string())
                .expect("Invalid redirect URL");
            config
                .into_client()
                .set_auth_type(oauth2::AuthType::RequestBody)
                .set_redirect_uri(redirect_url)
        }
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            output.set(response);
            return;
        }
    };

    const STATE_QUERY_PARAM_NAME: &str = "state";
    const CODE_QUERY_PARAM_NAME: &str = "code";

    let Some(param) = get_query_param(&url, STATE_QUERY_PARAM_NAME) else {
        eprintln!("missing '{STATE_QUERY_PARAM_NAME}' query parameter");
        let response = OutgoingResponse::new(500, &Headers::new(&[]));
        output.set(response);
        return;
    };

    let _state = CsrfToken::new(param);
    // TODO: check state with cached state and ensure equality

    let Some(param) = get_query_param(&url, CODE_QUERY_PARAM_NAME) else {
        eprintln!("missing '{CODE_QUERY_PARAM_NAME}' query parameter");
        let response = OutgoingResponse::new(500, &Headers::new(&[]));
        output.set(response);
        return;
    };

    let code_string = param;
    let code = AuthorizationCode::new(code_string.clone());

    async fn send_oauth_req(req: oauth2::HttpRequest) -> Result<oauth2::HttpResponse, SendError> {
        let mut builder = http::Request::builder()
            .method(req.method)
            .uri(req.url.as_str());

        for (name, value) in &req.headers {
            builder = builder.header(name, value);
        }

        let res = send::<_, http::Response<String>>(builder.body(req.body).unwrap()).await?;

        let (parts, body) = res.into_parts();

        Ok(oauth2::HttpResponse {
            status_code: parts.status,
            headers: parts.headers,
            body: body.into_bytes(),
        })
    }

    let result = client
        .exchange_code(code)
        .request_async(send_oauth_req)
        .await;

    match result {
        Ok(result) => {
            let access_token = serde_json::to_string(result.access_token())
                .unwrap()
                .replace("\"", "");
            let mut oauth_cookie = Cookie::new("access-token", access_token);
            oauth_cookie.set_same_site(Some(SameSite::Lax));
            oauth_cookie.set_http_only(true);
            oauth_cookie.set_path("/");

            let headers = Headers::new(&[
                ("Content-Type".to_string(), "text/plain".as_bytes().to_vec()),
                (
                    "Location".to_string(),
                    "http://127.0.0.1:3000/".as_bytes().to_vec(),
                ),
                (
                    "Set-Cookie".to_string(),
                    oauth_cookie.to_string().as_bytes().to_vec(),
                ),
            ]);

            let response = OutgoingResponse::new(301, &headers);
            output.set(response);
        }
        Err(error) => {
            eprintln!("error exchanging code for token with github: {error}");
            let response = OutgoingResponse::new(403, &Headers::new(&[]));
            output.set(response);
        }
    }
}

async fn authenticate(request: IncomingRequest, output: ResponseOutparam) {
    fn get_token(request: &IncomingRequest) -> Option<String> {
        let cookies: Vec<Vec<u8>> = request.headers().get(COOKIE.as_str());
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

    let token = match get_token(&request) {
        Some(token) => token,
        None => {
            eprintln!("no cookie found in incoming request");
            let headers =
                Headers::new(&[("Content-Type".to_string(), "text/html".as_bytes().to_vec())]);

            let response = OutgoingResponse::new(403, &headers);
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
                wasi::http::incoming_handler::handle(request, output.into_inner());
            } else {
                eprintln!("unauthenticated");
                let headers =
                    Headers::new(&[("Content-Type".to_string(), "text/html".as_bytes().to_vec())]);

                let response = OutgoingResponse::new(status.as_u16(), &headers);

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
