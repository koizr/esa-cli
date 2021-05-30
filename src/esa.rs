use std::collections::HashMap;
use std::fmt::{self, Debug};

use anyhow::Result;
use reqwest::{self, Client, ClientBuilder, Url};
use serde::Deserialize;
use thiserror::Error;

pub mod post;
pub mod team;

const BASE_URL: &str = "https://api.esa.io/v1";

pub struct Esa {
    client: Client,
    team: Team,
}
impl Esa {
    pub fn new(team: Team) -> Self {
        let client = ClientBuilder::new()
            .build()
            .expect("failed to build HTTP client");
        Esa { client, team }
    }

    pub async fn team(&self) -> Result<team::Team> {
        let response = self
            .client
            .get(format!("{}/teams/{}", BASE_URL, self.team.id))
            .bearer_auth(self.team.access_token.to_string())
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
            .get(format!("{}/teams/{}/posts/{}", BASE_URL, self.team.id, id))
            .bearer_auth(self.team.access_token.to_string())
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
            format!("{}/teams/{}/posts", BASE_URL, self.team.id).as_str(),
            query_string,
        )?;

        let response = self
            .client
            .get(url)
            .bearer_auth(self.team.access_token.to_string())
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

    pub async fn create_post(
        &self,
        post: post::PostContent,
        wip: bool,
        message: Option<String>,
    ) -> Result<post::NewPostCreated> {
        let new_post = post::NewPost {
            name: post.name,
            body_md: post.body_md,
            tags: post.tags,
            category: post.category,
            wip,
            message,
        };
        let response = self
            .client
            .post(format!("{}/teams/{}/posts", BASE_URL, self.team.id))
            .bearer_auth(self.team.access_token.to_string())
            .json(&new_post)
            .send()
            .await?;

        if response.status().is_success() {
            let post_created = response.json::<post::NewPostCreated>().await?;
            Ok(post_created)
        } else {
            let error = response.json::<ErrorResponse>().await?;
            Err(EsaError::Error(error))?
        }
    }

    pub async fn edit_post(&self, id: i32, post: &post::EditedPost) -> Result<post::PostEdited> {
        let response = self
            .client
            .patch(format!("{}/teams/{}/posts/{}", BASE_URL, self.team.id, id))
            .bearer_auth(self.team.access_token.to_string())
            .json(post)
            .send()
            .await?;

        if response.status().is_success() {
            let post_edited = response.json::<post::PostEdited>().await?;
            Ok(post_edited)
        } else {
            let error = response.json::<ErrorResponse>().await?;
            Err(EsaError::Error(error))?
        }
    }

    pub async fn delete_post(&self, id: i32) -> Result<()> {
        let response = self
            .client
            .delete(format!("{}/teams/{}/posts/{}", BASE_URL, self.team.id, id))
            .bearer_auth(self.team.access_token.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error = response.json::<ErrorResponse>().await?;
            Err(EsaError::Error(error))?
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Team {
    pub id: TeamId,
    pub access_token: AccessToken,
}

#[derive(Deserialize, PartialEq, Clone)]
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

impl Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AccessToken(**********)")
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
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
