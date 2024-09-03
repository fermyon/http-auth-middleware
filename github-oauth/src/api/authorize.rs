use super::OAuth2;
use oauth2::{basic, CsrfToken, Scope};
use spin_sdk::http::{Headers, OutgoingResponse, ResponseOutparam};

/// `authorize` kicks off the oauth flow constructing the authorization url and redirecting the client to github
/// to authorize the application to the user's profile.
pub async fn authorize(output: ResponseOutparam) {
    let client = match OAuth2::try_init() {
        Ok(config) => basic::BasicClient::new(config.client_id)
            .set_client_secret(config.client_secret)
            .set_auth_uri(config.auth_url)
            .set_token_uri(config.token_url)
            .set_redirect_uri(config.redirect_url)
            .set_auth_type(oauth2::AuthType::RequestBody),
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            let response = OutgoingResponse::new(Headers::new());
            response.set_status_code(500).unwrap();
            output.set(response);
            return;
        }
    };

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the user's email.
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // TODO: cache the csrf token for validation on callback

    let location = authorize_url.to_string().as_bytes().to_vec();
    let headers = Headers::new();
    headers.set(&"Location".to_string(), &[location]).unwrap();

    let response = OutgoingResponse::new(headers);
    response.set_status_code(301).unwrap();
    output.set(response);
}
