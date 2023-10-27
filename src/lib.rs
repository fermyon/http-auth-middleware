use cookie::Cookie;
use spin_sdk::http::{
    send, Headers, IncomingRequest, IncomingResponse, Method, OutgoingRequest, OutgoingResponse,
    ResponseOutparam,
};
use spin_sdk::http_component;

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

#[http_component]
async fn auth(req: IncomingRequest, out: ResponseOutparam) {
    let cookie = req
        .headers()
        .entries()
        .into_iter()
        .find_map(|(name, value)| {
            if name.to_owned() == http::header::COOKIE.to_string() {
                Some(value)
            } else {
                None
            }
        });

    let token = match cookie {
        Some(c) => get_token_from_cookie(&c),
        None => {
            let response = OutgoingResponse::new(401, &Headers::new(&[]));
            out.set(response);
            return;
        }
    };

    match validate_token(token).await {
        Ok(true) => {
            wasi::http::incoming_handler::handle(req, out.into_inner());
        }
        Ok(false) => {
            let response = OutgoingResponse::new(401, &Headers::new(&[]));
            out.set(response);
        }
        Err(error) => {
            println!("error {:?}", error);
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            out.set(response);
        }
    }
}

// #[http_component]
// async fn auth(req: IncomingRequest, out: ResponseOutparam) {
//     let token = get_token_from_cookie(req.headers());

//     match validate_token(token).await {
//         Ok(true) => {
//             wasi::http::incoming_handler::handle(req, out.into_inner());
//         }
//         Ok(false) => {
//             let response = OutgoingResponse::new(401, &Headers::new(&[]));
//             out.set(response);
//         }
//         Err(_error) => {
//             let response = OutgoingResponse::new(500, &Headers::new(&[]));
//             out.set(response);
//         }
//     }
// }

async fn validate_token(token: Option<String>) -> anyhow::Result<bool> {
    if token.is_none() {
        return Ok(false);
    }
    let auth = format!("Bearer {:?}", token);

    println!("before outgoing request");
    let req = OutgoingRequest::new(
        &Method::Get,
        Some("/user"),
        Some(&spin_sdk::http::Scheme::Https),
        Some("api.github.com"), // authority
        &Headers::new(&[
            (
                "Accept".to_string(),
                b"application/vnd.github+json".to_vec(),
            ),
            ("Authorization".to_string(), auth.as_bytes().to_vec()),
            ("User-Agent".to_string(), b"spin-triage".to_vec()),
            ("X-GitHub-Api-Version".to_string(), b"2022-11-28".to_vec()),
        ]),
    );

    // let request: spin_sdk::http::Request = req.try_into()?;
    let res: IncomingResponse = send(req).await?;
    let status = res.status();
    println!("status {:?}", status);

    // println!("res {:?}", res.try_into()?);

    if status == 200 {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_token_from_cookie(cookie: &[u8]) -> Option<String> {
    if let Ok(cookies) = Cookie::parse(String::from_utf8_lossy(cookie)) {
        // TODO handle to_str error
        let (name, val) = cookies.name_value();
        if name == "oauth-token" {
            return Some(val.to_string());
        }
    }
    None
}
