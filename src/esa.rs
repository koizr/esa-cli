use reqwest::{self, Client, ClientBuilder};
use serde::Deserialize;

const BASE_URL: &str = "https://api.esa.io/v1";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct Esa {
    client: Client,
}
impl Esa {
    pub fn new() -> Self {
        let client = ClientBuilder::new().build().unwrap();
        Esa { client }
    }

    pub async fn team(&self, id: TeamId, token: String) -> Result<Team> {
        let team = self
            .client
            .get(format!("{}/teams/{}", BASE_URL, id.0))
            .bearer_auth(token)
            .send()
            .await?
            .json::<Team>()
            .await?;
        Ok(team)
    }
}

impl Default for Esa {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TeamId(String);
impl TeamId {
    pub fn new(id: String) -> Self {
        TeamId(id)
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

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
