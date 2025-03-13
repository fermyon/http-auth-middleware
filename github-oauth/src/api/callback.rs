use super::OAuth2;
use crate::wasi::http::types::{Body, ErrorCode, Headers, Request, Response};
use crate::wit_stream;
use cookie::{Cookie, SameSite};
use oauth2::{basic, AuthorizationCode, CsrfToken, TokenResponse};
use url::Url;
use wit_bindgen_rt::async_support::futures::{SinkExt, StreamExt};

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
            let response = Response::new(Headers::new(), None);
            response.set_status_code(500).unwrap();
            return Ok(response);
        }
    };

    let (code, _state) = match get_code_and_state_param(&url) {
        Ok((code, state)) => (code, state),
        Err(error) => {
            eprintln!("error retrieving required query parameters: {error}");
            let response = Response::new(Headers::new(), None);
            response.set_status_code(500).unwrap();
            return Ok(response);
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

            let headers = Headers::new();
            headers
                .set("Content-Type", &[b"text/plain".to_vec()])
                .unwrap();
            headers
                .set("Location", &[location.to_string().into_bytes()])
                .unwrap();
            headers
                .set("Set-Cookie", &[oauth_cookie.to_string().into_bytes()])
                .unwrap();

            let response = Response::new(headers, None);
            response.set_status_code(301).unwrap();
            Ok(response)
        }
        Err(error) => {
            eprintln!("error exchanging code for token with github: {error}");
            let response = Response::new(Headers::new(), None);
            response.set_status_code(403).unwrap();
            Ok(response)
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
    let wasi_headers = Headers::new();
    for (name, value) in oauth_req.headers() {
        wasi_headers
            .set(name.to_string().as_str(), &[value.as_bytes().to_vec()])
            .unwrap();
    }

    let oauth_body = oauth_req.body().clone();
    let wasi_body = if oauth_body.is_empty() {
        None
    } else {
        let (mut writer, reader) = wit_stream::new();
        wit_bindgen_rt::async_support::spawn(async move { writer.send(oauth_body).await.unwrap() });
        Some(Body::new(reader).0)
    };

    let wasi_request = Request::new(wasi_headers, wasi_body, None);
    wasi_request
        .set_method(&wasi_method(oauth_req.method()))
        .unwrap();
    wasi_request
        .set_scheme(oauth_req.uri().scheme().map(wasi_scheme))
        .unwrap();
    wasi_request
        .set_authority(oauth_req.uri().authority().map(|a| a.as_str()))
        .unwrap();
    wasi_request
        .set_path_with_query(oauth_req.uri().path_and_query().map(|pq| pq.as_str()))
        .unwrap();

    let wasi_response = crate::wasi::http::handler::handle(wasi_request).await?;

    let oauth_response_body = body_to_vec(&wasi_response).await.map_err(as_code)?;

    let mut oauth_response = oauth2::HttpResponse::new(oauth_response_body);
    *oauth_response.status_mut() =
        oauth2::http::StatusCode::from_u16(wasi_response.status_code()).unwrap();

    for (name, value) in wasi_response.headers().entries() {
        oauth_response.headers_mut().insert(
            oauth2::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            oauth2::http::HeaderValue::from_bytes(&value).unwrap(),
        );
    }

    Ok(oauth_response)
}

async fn body_to_vec(response: &crate::wasi::http::types::Response) -> anyhow::Result<Vec<u8>> {
    let Some(body) = response.body() else {
        return Ok(vec![]);
    };

    let (mut reader, _efut) = body
        .stream()
        .map_err(|_| anyhow::anyhow!("failed to stream body"))?;

    let mut vector = vec![];

    loop {
        match reader.next().await {
            None => break,
            Some(chunk) => {
                let mut chunk = chunk?;
                vector.append(&mut chunk);
            }
        }
    }

    Ok(vector)
}

fn as_code(e: anyhow::Error) -> ErrorCode {
    ErrorCode::InternalError(Some(e.to_string()))
}

fn wasi_method(method: &oauth2::http::Method) -> crate::wasi::http::types::Method {
    match *method {
        oauth2::http::Method::GET => crate::wasi::http::types::Method::Get,
        oauth2::http::Method::POST => crate::wasi::http::types::Method::Post,
        _ => panic!("unexpected OAuth method {}", method),
    }
}

fn wasi_scheme(scheme: &oauth2::http::uri::Scheme) -> &'static crate::wasi::http::types::Scheme {
    // TODO: better way?
    match scheme.as_str() {
        "http" => &crate::wasi::http::types::Scheme::Http,
        "https" => &crate::wasi::http::types::Scheme::Https,
        _ => panic!("unexpected OAuth scheme {}", scheme),
    }
}
