use std::collections::HashMap;
use std::fmt;

use anyhow::Result;
use reqwest::{self, Client, ClientBuilder, Url};
use serde::Deserialize;
use thiserror::Error;

pub mod post;
pub mod team;

const BASE_URL: &str = "https://api.esa.io/v1";

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

    pub async fn team(&self) -> Result<team::Team> {
        let response = self
            .client
            .get(format!("{}/teams/{}", BASE_URL, self.team_id))
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?;
        if response.status().is_success() {
            let team = response.json::<team::Team>().await?;
            Ok(team)
        } else {
            let error = response.json::<ErrorResponse>().await?;
            Err(EsaError::Error(error))?
        }
    }

    pub async fn post(&self, id: i32) -> Result<post::Post> {
        let response = self
            .client
            .get(format!("{}/teams/{}/posts/{}", BASE_URL, self.team_id, id))
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            let post = response.json::<post::Post>().await?;
            Ok(post)
        } else {
            let error = response.json::<ErrorResponse>().await?;
            Err(EsaError::Error(error))?
        }
    }

    pub async fn posts(&self, query: post::SearchQuery) -> Result<post::Posts> {
        let mut query_string: HashMap<&str, String> = HashMap::new();
        if let Some(q) = query.q {
            query_string.insert("q", q);
        }
        if let Some(include) = query.include {
            let include: Vec<String> = include.into_iter().map(Into::into).collect();
            query_string.insert("include", include.join(","));
        }
        if let Some(sort) = query.sort {
            let (s, o) = sort.into();
            query_string.insert("sort", s);
            query_string.insert("order", o);
        }

        let url = Url::parse_with_params(
            format!("{}/teams/{}/posts", BASE_URL, self.team_id).as_str(),
            query_string,
        )?;

        let response = self
            .client
            .get(url)
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?;
        if response.status().is_success() {
            let posts = response.json::<post::Posts>().await?;
            Ok(posts)
        } else {
            let error = response.json::<ErrorResponse>().await?;
            Err(EsaError::Error(error))?
        }
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
pub struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Debug, Error)]
pub enum EsaError {
    #[error("error: {}, message: {}", .0.error, .0.message)]
    Error(ErrorResponse),
}
