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
        let client_secret = ClientSecret::new(std::env::var("CLIENT_SECRET")?);
        let client_id = ClientId::new(std::env::var("CLIENT_ID")?);
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
