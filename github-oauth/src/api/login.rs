use crate::wasi::http::types::{Body, ErrorCode, Headers, Response};
use crate::wit_stream;

use wit_bindgen_rt::async_support::futures::SinkExt;

/// `login` returns the login page.
pub async fn login() -> Result<Response, ErrorCode> {
    const LOGIN_HTML: &[u8] = include_bytes!("../../login.html"); // TODO: this shouldn't be included statically.

    let headers = Headers::new();

    if let Err(err) = headers.set("content-type", &[b"text/html".to_vec()]) {
        eprintln!("error setting content-type header: {err}");
        let response = Response::new(headers, None);
        response.set_status_code(500).unwrap();
        return Ok(response);
    }

    let (mut writer, reader) = wit_stream::new();

    wit_bindgen_rt::async_support::spawn(async move {
        writer.send(LOGIN_HTML.to_vec()).await.unwrap();
    });

    let (body, _err_fut) = Body::new(reader);
    let response = Response::new(headers, Some(body));
    response.set_status_code(200).unwrap();

    Ok(response)
}
