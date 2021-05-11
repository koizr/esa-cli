use std::fmt;

use reqwest::{self, Client, ClientBuilder};
use serde::Deserialize;

const BASE_URL: &str = "https://api.esa.io/v1";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct Esa {
    client: Client,
    team_id: TeamId,
    access_token: AccessToken,
}
impl Esa {
    pub fn new(team_id: TeamId, access_token: AccessToken) -> Self {
        let client = ClientBuilder::new().build().unwrap();
        Esa {
            client,
            team_id,
            access_token,
        }
    }

    pub async fn team(&self) -> Result<Team> {
        let team = self
            .client
            .get(format!("{}/teams/{}", BASE_URL, self.team_id))
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?
            .json::<Team>()
            .await?;
        Ok(team)
    }
}

pub struct AccessToken(String);
impl AccessToken {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}
impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct TeamId(String);
impl TeamId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}
impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize)]
pub struct Team {
    pub name: String,
    pub privacy: TeamPrivacy,
    pub description: String,
    pub icon: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub enum TeamPrivacy {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "open")]
    Open,
}
