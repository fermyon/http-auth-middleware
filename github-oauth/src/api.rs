use anyhow::Context;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

pub use authenticate::authenticate;
pub use authorize::authorize;
pub use callback::callback;
pub use login::login;

mod authenticate;
mod authorize;
mod callback;
mod login;

pub struct OAuth2 {
    pub client_secret: ClientSecret,
    pub client_id: ClientId,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
    pub redirect_url: RedirectUrl,
}

const AUTH_CALLBACK_URL: Option<&'static str> = option_env!("AUTH_CALLBACK_URL");

impl OAuth2 {
    pub fn try_init() -> anyhow::Result<Self> {
        let client_secret_env = option_env!("CLIENT_SECRET");
        let client_id_env = option_env!("CLIENT_ID");

        let (client_secret, client_id) = if !cfg!(feature = "compile-time-secrets") {
            (std::env::var("CLIENT_SECRET")?, std::env::var("CLIENT_ID")?)
        } else {
            (
                client_secret_env
                    .context("CLIENT_SECRET was not configured at build time")?
                    .to_string(),
                client_id_env
                    .context("CLIENT_ID was not configured at build time")?
                    .to_string(),
            )
        };

        let client_secret = ClientSecret::new(client_secret);
        let client_id = ClientId::new(client_id);
        let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?;
        let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?;

        let redirect_url = match std::env::var("AUTH_CALLBACK_URL") {
            Ok(runtime_env) => RedirectUrl::new(runtime_env)?,
            Err(_) => RedirectUrl::new(
                AUTH_CALLBACK_URL
                    .unwrap_or("http://127.0.0.1:3000/login/callback")
                    .to_string(),
            )?,
        };

        Ok(OAuth2 {
            client_secret,
            client_id,
            token_url,
            auth_url,
            redirect_url,
        })
    }
}
