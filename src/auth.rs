use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde_json::{Value, from_value};
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::api::cache_to_disk;

static SCOPES: [&str; 2] = ["service:leagues", "service:cxapi"];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientToken {
    pub access_token: String,
    pub expires_in: Option<String>,
    pub token_type: String,
    pub username: String,
    pub sub: String,
    pub scope: String,
}

impl ClientToken {
    pub fn from_authorized_file(filepath: &Path) -> Result<Box<Self>> {
        let file = File::open(filepath)?;
        let token: ClientToken = serde_json::from_reader(file)?;
        Ok(Box::new(token))
    }

    pub fn new() -> Box<Self> {
        Box::new(ClientToken::default())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub scope: String,
}

impl ClientCredentials {
    pub fn new() -> Box<Self> {
        Box::new(ClientCredentials::default())
    }

    pub fn from_secrets_file(filepath: &Path) -> Result<Box<Self>> {
        let file = File::open(filepath)?;
        let creds: ClientCredentials = serde_json::from_reader(file)?;
        Ok(Box::new(creds))
    }

    pub async fn get_client_credentials_grant(&self) -> Result<Box<ClientToken>> {
        // Since I probably won't hit this path much, we can just build a client
        let client = reqwest::Client::builder()
            .user_agent("Oauth poeflipfinder/0.0.1 (contact: camiam144@gmail.com)")
            .build()?;
        let url = Url::parse("https://www.pathofexile.com/oauth/token").unwrap();

        let response = client
            .post(url)
            .form(self)
            .send()
            .await?
            .error_for_status()?;

        response.json().await.context("Couldn't parse new token")
    }
}

pub async fn get_cxapi_cred() -> Result<Box<ClientToken>> {
    let mut token: Option<Box<ClientToken>> = None;

    let creds_path = Path::new("src/creds.json");
    let token_path = Path::new("src/.token");

    if token_path.is_file() {
        token = Some(ClientToken::from_authorized_file(token_path)?);
    }
    // This needs a check if the credential is expired or if it's invalidated
    // I don't have to worry about refresh tokens with this type of credential
    // If I can't get a valid token I likely need a new secret from the website.
    if token.is_none() {
        // Get the auth and save it
        println!("Getting new token");
        let creds: Box<ClientCredentials> = ClientCredentials::from_secrets_file(creds_path)?;
        token = Some(creds.get_client_credentials_grant().await?);
        cache_to_disk(&token, token_path)?;
    }

    token.context("Couldn't get a valid token")
}
