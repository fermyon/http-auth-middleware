use super::OAuth2;
use crate::sdk::http::{RequestBuilder, ResponseBuilder};
use crate::wasi::http::types::{ErrorCode, Response};
use cookie::{Cookie, SameSite};
use oauth2::{basic, AuthorizationCode, CsrfToken, TokenResponse};
use url::Url;

pub async fn callback(url: Url) -> Result<Response, ErrorCode> {
    let client = match OAuth2::try_init() {
        Ok(config) => basic::BasicClient::new(config.client_id)
            .set_client_secret(config.client_secret)
            .set_auth_uri(config.auth_url)
            .set_token_uri(config.token_url)
            .set_redirect_uri(config.redirect_url)
            .set_auth_type(oauth2::AuthType::RequestBody),
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            return ResponseBuilder::new().with_status_code(5000).empty();
        }
    };

    let (code, _state) = match get_code_and_state_param(&url) {
        Ok((code, state)) => (code, state),
        Err(error) => {
            eprintln!("error retrieving required query parameters: {error}");
            return ResponseBuilder::new().with_status_code(5000).empty();
        }
    };

    // TODO: check state with cached state and ensure equality

    let result = client
        .exchange_code(code)
        .request_async(&oauth_http_client)
        .await;

    let mut location = client.redirect_uri().unwrap().url().clone();
    location.set_path("");
    match result {
        Ok(result) => {
            let access_token = serde_json::to_string(result.access_token())
                .unwrap()
                .replace("\"", "");

            let mut oauth_cookie = Cookie::new("access-token", access_token);
            oauth_cookie.set_same_site(Some(SameSite::Lax));
            oauth_cookie.set_http_only(true);
            oauth_cookie.set_path("/");

            ResponseBuilder::new()
                .with_status_code(301)
                .with_header("Content-Type", "text/plain")?
                .with_header("Location", &location)?
                .with_header("Set-Cookie", oauth_cookie.to_string())?
                .empty()
        }
        Err(error) => {
            eprintln!("error exchanging code for token with github: {error}");
            ResponseBuilder::new().with_status_code(403).empty()
        }
    }
}

fn get_code_and_state_param(url: &Url) -> anyhow::Result<(AuthorizationCode, CsrfToken)> {
    fn get_query_param(url: &Url, param: &str) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| key == param)
            .map(|(_, value)| value.into_owned())
    }

    const STATE_QUERY_PARAM_NAME: &str = "state";
    const CODE_QUERY_PARAM_NAME: &str = "code";

    let Some(param) = get_query_param(url, STATE_QUERY_PARAM_NAME) else {
        anyhow::bail!("missing '{STATE_QUERY_PARAM_NAME}' query parameter");
    };

    let state = CsrfToken::new(param);

    let Some(param) = get_query_param(url, CODE_QUERY_PARAM_NAME) else {
        anyhow::bail!("missing '{CODE_QUERY_PARAM_NAME}' query parameter");
    };

    let code = AuthorizationCode::new(param);

    Ok((code, state))
}

async fn oauth_http_client(
    oauth_req: oauth2::HttpRequest,
) -> Result<oauth2::HttpResponse, ErrorCode> {
    let wasi_request = RequestBuilder::from_hyper(oauth_req);

    let wasi_response = crate::wasi::http::handler::handle(wasi_request).await?;

    crate::sdk::http::as_hyper(wasi_response).await
}
