use super::OAuth2;
use crate::response;

use cookie::{Cookie, SameSite};
use oauth2::{basic, AuthorizationCode, CsrfToken, TokenResponse};
use spin_sdk::http_wasip3::{send, EmptyBody, FullBody, IntoResponse};
use url::Url;

pub async fn callback(url: &http::Uri) -> impl IntoResponse {
    let Ok(url) = url::Url::parse(&url.to_string()) else {
        return response::internal_server_error().into_response();
    };

    let client = match OAuth2::try_init() {
        Ok(config) => basic::BasicClient::new(config.client_id)
            .set_client_secret(config.client_secret)
            .set_auth_uri(config.auth_url)
            .set_token_uri(config.token_url)
            .set_redirect_uri(config.redirect_url)
            .set_auth_type(oauth2::AuthType::RequestBody),
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            return response::internal_server_error().into_response();
        }
    };

    let (code, _state) = match get_code_and_state_param(&url) {
        Ok((code, state)) => (code, state),
        Err(error) => {
            eprintln!("error retrieving required query parameters: {error}");
            return response::internal_server_error().into_response();
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

            http::Response::builder()
                .status(301)
                .header("Content-Type", "text/plain")
                .header("Location", location.as_str())
                .header("Set-Cookie", oauth_cookie.to_string())
                .body(EmptyBody::new())
                .into_response()
        }
        Err(error) => {
            eprintln!("error exchanging code for token with github: {error}");
            response::forbidden().into_response()
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
) -> Result<oauth2::HttpResponse, spin_sdk::http_wasip3::Error> {
    use spin_sdk::http_wasip3::body::IncomingBodyExt;

    let wasi_req = oauth_req.map(|bod| FullBody::new(bytes::Bytes::from_owner(bod)));
    let wasi_response = send(wasi_req).await?;
    let (parts, body) = wasi_response.into_parts();
    let sync_body = body.bytes().await?.to_vec();
    let oauth_response = http::Response::from_parts(parts, sync_body);
    Ok(oauth_response)
}
