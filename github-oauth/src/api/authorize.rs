use super::OAuth2;
use oauth2::{CsrfToken, RedirectUrl, Scope};
use spin_sdk::http::{Headers, OutgoingResponse, ResponseOutparam};

/// `authorize` kicks off the oauth flow constructing the authorization url and redirecting the client to github
/// to authorize the application to the user's profile.
pub async fn authorize(output: ResponseOutparam) {
    let client = match OAuth2::try_init() {
        Ok(config) => {
            let redirect_url = RedirectUrl::new("http://127.0.0.1:3000/login/callback".to_string())
                .expect("Invalid redirect URL");
            config
                .into_client()
                .set_auth_type(oauth2::AuthType::RequestBody)
                .set_redirect_uri(redirect_url)
        }
        Err(error) => {
            eprintln!("failed to initialize oauth client: {error}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
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
    let headers = Headers::new(&[("Location".to_string(), location)]);
    let response = OutgoingResponse::new(301, &headers);
    output.set(response);
}
