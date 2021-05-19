use std::collections::HashMap;
use std::fmt;

use reqwest::{self, Client, ClientBuilder, Url};

pub mod post;
pub mod team;

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

    pub async fn team(&self) -> Result<team::Team> {
        let team = self
            .client
            .get(format!("{}/teams/{}", BASE_URL, self.team_id))
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?
            .json::<team::Team>()
            .await?;
        Ok(team)
    }

    pub async fn post(&self, id: i32) -> Result<post::Post> {
        let post = self
            .client
            .get(format!("{}/teams/{}/posts/{}", BASE_URL, self.team_id, id))
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?
            .json::<post::Post>()
            .await?;
        Ok(post)
    }

    pub async fn posts(&self, query: post::SearchQuery) -> Result<post::Posts> {
        let mut query_string: HashMap<String, String> = HashMap::new();
        if let Some(q) = query.q {
            query_string.insert("q".to_string(), q);
        }
        if let Some(include) = query.include {
            let include: Vec<String> = include.into_iter().map(Into::into).collect();
            query_string.insert("include".to_string(), include.join(","));
        }
        if let Some(sort) = query.sort {
            let (s, o) = sort.into();
            query_string.insert("sort".to_string(), s);
            query_string.insert("order".to_string(), o);
        }

        let url = Url::parse_with_params(
            format!("{}/teams/{}/posts", BASE_URL, self.team_id).as_str(),
            query_string,
        )?;

        let posts = self
            .client
            .get(url)
            .bearer_auth(self.access_token.to_string())
            .send()
            .await?
            .json::<post::Posts>()
            .await?;
        Ok(posts)
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
