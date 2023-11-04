use anyhow::Context;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};

pub use authenticate::authenticate;
pub use authorize::authorize;
pub use callback::callback;
pub use login::login;

mod authenticate;
mod authorize;
mod callback;
mod login;

pub struct OAuth2 {
    client_secret: ClientSecret,
    client_id: ClientId,
    auth_url: AuthUrl,
    token_url: TokenUrl,
}

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

        Ok(OAuth2 {
            client_secret,
            client_id,
            token_url,
            auth_url,
        })
    }

    pub fn into_client(self) -> BasicClient {
        BasicClient::new(
            self.client_id,
            Some(self.client_secret),
            self.auth_url,
            Some(self.token_url),
        )
    }
}
