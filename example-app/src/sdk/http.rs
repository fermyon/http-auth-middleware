// This subsets the `sdk` module from the main sample,
// including only the parts relevant to the back end.
// Once WASI Preview 3 is stable, the Spin SDK will provide a common
// home for appropriate helpers and wrappers such as these.

use wit_bindgen_rt::async_support::futures::SinkExt;
use wit_bindgen_rt::async_support::StreamWriter;

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
