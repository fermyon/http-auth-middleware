use spin_sdk::http::{Headers, OutgoingResponse, ResponseOutparam};

/// `login` returns the login page.
pub async fn login(output: ResponseOutparam) {
    const LOGIN_HTML: &[u8] = include_bytes!("../../login.html"); // TODO: this shouldn't be included statically.

    let headers = Headers::new(&[("content-type".to_string(), b"text/html".to_vec())]);

    let response = OutgoingResponse::new(200, &headers);

    if let Err(error) = output.set_with_body(response, LOGIN_HTML.to_vec()).await {
        eprintln!("error send login page: {error}");
    }
}
