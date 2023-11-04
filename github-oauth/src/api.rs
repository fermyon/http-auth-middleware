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

pub const AUTH_CALLBACK_URL: Option<&'static str> = option_env!("AUTH_CALLBACK_URL");

impl OAuth2 {
    pub fn try_init() -> anyhow::Result<Self> {
        let client_id_env = "";
        let client_secret_env = "";

        #[cfg(feature = "compile-time-secrets")]
        let (client_secret_env, client_id_env) = (
            env!("CLIENT_SECRET").to_string(),
            env!("CLIENT_ID").to_string(),
        );

        let (client_secret, client_id) = if !cfg!(feature = "compile-time-secrets") {
            (std::env::var("CLIENT_SECRET")?, std::env::var("CLIENT_ID")?)
        } else {
            (client_secret_env.to_string(), client_id_env.to_string())
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
