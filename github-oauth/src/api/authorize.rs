use super::OAuth2;
use crate::{
    sdk::http::ResponseBuilder,
    wasi::http::types::{ErrorCode, Response},
};
use oauth2::{basic, CsrfToken, Scope};

/// `authorize` kicks off the oauth flow constructing the authorization url and redirecting the client to github
/// to authorize the application to the user's profile.
pub async fn authorize() -> Result<Response, ErrorCode> {
    let client = match OAuth2::try_init() {
        Ok(config) => basic::BasicClient::new(config.client_id)
            .set_client_secret(config.client_secret)
            .set_auth_uri(config.auth_url)
            .set_token_uri(config.token_url)
            .set_redirect_uri(config.redirect_url)
            .set_auth_type(oauth2::AuthType::RequestBody),
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            return ResponseBuilder::new().with_status_code(500).empty();
        }
    };

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the user's email.
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // TODO: cache the csrf token for validation on callback

    ResponseBuilder::new()
        .with_header("Location", &authorize_url)?
        .with_status_code(301)
        .empty()
}
