use anyhow::{Context, Ok, Result, anyhow};
use reqwest::Url;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::{fs::File, io};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum AuthorizedScopes {
    #[serde(rename = "service:cxapi")]
    Cxapi,
    #[serde(rename = "service:leagues")]
    Leagues,
}
impl FromStr for AuthorizedScopes {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "service:cxapi" => std::result::Result::Ok(AuthorizedScopes::Cxapi),
            "service:leagues" => std::result::Result::Ok(AuthorizedScopes::Leagues),
            _ => Err("Unknown scope"),
        }
    }
}
impl fmt::Display for AuthorizedScopes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cxapi => write!(f, "service:cxapi"),
            Self::Leagues => write!(f, "service:leagues"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientToken {
    pub access_token: String,
    pub expires_in: Option<String>,
    pub token_type: String,
    pub username: String,
    pub sub: String,
    pub scope: AuthorizedScopes,
}

impl ClientToken {
    pub fn from_authorized_file(
        filepath: &Path,
        scope: &AuthorizedScopes,
    ) -> Result<Option<Box<Self>>> {
        let file = File::open(filepath);
        if file.is_err() {
            return Ok(None);
        }
        let reader = BufReader::new(file?);

        let tokens: Vec<ClientToken> = serde_json::from_reader(reader)?;
        let token: Vec<&ClientToken> = tokens.iter().filter(|t| t.scope == *scope).collect();

        match token.len() {
            1 => Ok(Some(Box::new(token[0].clone()))),
            0 => Ok(None),
            _ => Err(anyhow!(
                "Too many tokens match this scope. No more than 1 should be saved"
            )),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub scopes: Vec<AuthorizedScopes>,
}

impl ClientCredentials {
    pub fn from_secrets_file(filepath: &Path) -> Result<Box<Self>> {
        let file = File::open(filepath)?;
        let creds: ClientCredentials = serde_json::from_reader(file)?;
        Ok(Box::new(creds))
    }

    pub async fn get_client_credentials_grant(
        &self,
        scope: &AuthorizedScopes,
    ) -> Result<Box<ClientToken>> {
        // Since I probably won't hit this path much, we can just build a client
        let client = reqwest::Client::builder()
            .user_agent("Oauth poeflipfinder/0.0.1 (contact: camiam144@gmail.com)")
            .build()?;
        let url = Url::parse("https://www.pathofexile.com/oauth/token")?;

        let mut form = HashMap::new();

        form.insert("client_id", self.client_id.clone());
        form.insert("client_secret", self.client_secret.clone());
        form.insert("grant_type", self.grant_type.clone());
        form.insert("scope", scope.to_string());

        let response = client
            .post(url)
            .form(&form)
            .send()
            .await?
            .error_for_status()?;

        response.json().await.context("Couldn't parse new token")
    }
}

pub async fn get_api_token(scope: &AuthorizedScopes) -> Result<Box<ClientToken>> {
    let creds_path = Path::new("src/creds.json");
    let token_path = Path::new("src/.tokens");

    let mut token = ClientToken::from_authorized_file(token_path, scope)?;

    // This needs a check if the credential is expired or if it's invalidated
    // I don't have to worry about refresh tokens with this type of credential
    // If I can't get a valid token I likely need a new secret from the website.
    if token.is_none() {
        // Get the auth and save it
        let creds: Box<ClientCredentials> = ClientCredentials::from_secrets_file(creds_path)?;
        token = Some(creds.get_client_credentials_grant(scope).await?);
        // This leads me to believe I shouldn't have worked with boxes...
        cache_tokens(token.clone().unwrap().as_ref(), token_path)?;
    }

    token.context("Couldn't get a valid token")
}

/// Cache new auth tokens to disk so we don't have to keep getting new ones
pub fn cache_tokens(data: &ClientToken, file_path: &Path) -> Result<()> {
    let mut tokens: Vec<ClientToken>;

    {
        let maybe_file = File::open(file_path);
        if let core::result::Result::Ok(current_data) = maybe_file {
            let reader = BufReader::new(current_data);
            tokens = serde_json::from_reader(reader)?;
        } else {
            tokens = Vec::new();
        }
    }
    tokens.push(data.clone());

    let file = File::create(file_path)?;
    let writer = io::BufWriter::new(file);

    serde_json::to_writer(writer, &tokens)?;
    Ok(())
}
