use wit_bindgen_rt::async_support::futures::{SinkExt, StreamExt};
use wit_bindgen_rt::async_support::StreamWriter;

use crate::wasi::http::types::Request;
use crate::{
    wasi::http::types::{Body, ErrorCode, HeaderError, Headers, Response},
    wit_stream,
};

pub struct ResponseBuilder {
    status_code: u16,
    headers: Headers,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: Headers::new(),
        }
    }

    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn with_header(
        self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<Self, ErrorCode> {
        self.with_binary_header(name, value.as_ref().as_bytes())
    }

    pub fn with_binary_header(
        self,
        name: impl AsRef<str>,
        value: impl AsRef<[u8]>,
    ) -> Result<Self, ErrorCode> {
        self.headers
            .set(name.as_ref(), &[value.as_ref().to_vec()])
            .map_err(header_error_code)?;
        Ok(self)
    }

    pub fn empty(self) -> Result<Response, ErrorCode> {
        let response = Response::new(self.headers, None);
        response
            .set_status_code(self.status_code)
            .map_err(|_| ErrorCode::InternalError(Some("Invalid status code".to_owned())))?;
        Ok(response)
    }

    pub fn text(self, body: impl AsRef<str>) -> Result<Response, ErrorCode> {
        self.binary(body.as_ref().as_bytes().to_vec())
    }

    pub fn binary(self, body: impl Into<Vec<u8>>) -> Result<Response, ErrorCode> {
        let body = body.into();

        let (response, mut writer) = self.streaming()?;

        wit_bindgen_rt::async_support::spawn(async move {
            writer.send(body).await.unwrap();
        });

        Ok(response)
    }

    pub fn streaming(self) -> Result<(Response, StreamWriter<u8>), ErrorCode> {
        let (writer, reader) = wit_stream::new();
        let response_body = Body::new(reader).0;

        let response = Response::new(self.headers, Some(response_body));
        response
            .set_status_code(self.status_code)
            .map_err(|_| ErrorCode::InternalError(Some("Invalid status code".to_owned())))?;
        Ok((response, writer))
    }
}

fn header_error_code(e: HeaderError) -> ErrorCode {
    let message = match e {
        HeaderError::InvalidSyntax => "Invalid header syntax",
        HeaderError::Forbidden => "Forbidden header",
        HeaderError::Immutable => "Attempted to modify immutable header",
    };
    ErrorCode::InternalError(Some(message.to_owned()))
}

pub struct RequestBuilder {}

impl RequestBuilder {
    pub fn from_hyper(oauth_req: http::Request<Vec<u8>>) -> Request {
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
            wit_bindgen_rt::async_support::spawn(
                async move { writer.send(oauth_body).await.unwrap() },
            );
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

        wasi_request
    }
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

pub async fn as_hyper(wasi_response: Response) -> Result<http::Response<Vec<u8>>, ErrorCode> {
    let hyper_response_body = body_to_vec(&wasi_response).await.map_err(as_code)?;

    let mut hyper_response = oauth2::HttpResponse::new(hyper_response_body);
    *hyper_response.status_mut() =
        oauth2::http::StatusCode::from_u16(wasi_response.status_code()).unwrap();

    for (name, value) in wasi_response.headers().entries() {
        hyper_response.headers_mut().insert(
            oauth2::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            oauth2::http::HeaderValue::from_bytes(&value).unwrap(),
        );
    }

    Ok(hyper_response)
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
