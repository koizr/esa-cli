use std::fmt;

use chrono::{prelude::Local, DateTime};
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

    pub async fn post(&self, id: i32) -> Result<Post> {
        let post = self
            .client
            .get(format!("{}/teams/{}/posts/{}", BASE_URL, self.team_id, id))
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?
            .json::<Post>()
            .await?;
        Ok(post)
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

#[derive(Debug, Deserialize)]
pub struct Post {
    pub number: i32,
    pub name: String,
    pub full_name: String,
    pub wip: bool,
    pub body_md: String,
    pub body_html: String,
    pub created_at: DateTime<Local>,
    pub message: String,
    pub url: String,
    pub updated_at: DateTime<Local>,
    pub tags: Vec<String>,
    pub category: String,
    pub revision_number: i32,
    pub created_by: Writer,
    pub updated_by: Writer,
    pub kind: PostKind,
    pub comments_count: i32,
    pub tasks_count: i32,
    pub done_tasks_count: i32,
    pub stargazers_count: i32,
    pub watchers_count: i32,
    pub star: bool,
    pub watch: bool,
    // TODO: comments, stargazers を追加する
}

#[derive(Debug, Deserialize)]
pub struct Writer {
    pub myself: bool,
    pub name: String,
    pub screen_name: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub enum PostKind {
    #[serde(rename = "stock")]
    Stock,
    #[serde(rename = "flow")]
    Flow,
}
