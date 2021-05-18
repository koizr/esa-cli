use std::collections::HashMap;
use std::fmt;

use chrono::{prelude::Local, DateTime};
use reqwest::{self, Client, ClientBuilder, Url};
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

    pub async fn posts(&self, query: PostsQuery) -> Result<Posts> {
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
            .json::<Posts>()
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

#[derive(Debug, Deserialize)]
pub struct Posts {
    pub posts: Vec<Post>,
    pub prev_page: Option<i32>,
    pub next_page: Option<i32>,
    pub total_count: i32,
    pub page: i32,
    pub per_page: i32,
    pub max_per_page: i32,
}

pub struct PostsQuery {
    pub q: Option<String>,
    pub include: Option<Vec<PostInclude>>,
    pub sort: Option<PostSort>,
}

impl PostsQuery {
    pub fn new(
        q: Option<String>,
        include: Option<Vec<PostInclude>>,
        sort: Option<PostSort>,
    ) -> Self {
        PostsQuery { q, include, sort }
    }
}

pub enum PostInclude {
    /// スターを含む
    Stargazers,
    /// コメントを含む
    Comments,
    /// コメントに対するスターを含む
    CommentStargazers,
    /// なし
    None,
}

impl From<String> for PostInclude {
    fn from(s: String) -> Self {
        match s.as_str() {
            "stargazers" => Self::Stargazers,
            "comments" => Self::Comments,
            "comments.stargazers" => Self::CommentStargazers,
            _ => Self::None,
        }
    }
}

impl Into<String> for PostInclude {
    fn into(self) -> String {
        match self {
            Self::Stargazers => "stargazers".to_string(),
            Self::Comments => "comments".to_string(),
            Self::CommentStargazers => "comments.stargazers".to_string(),
            Self::None => "".to_string(),
        }
    }
}

pub enum PostSort {
    /// 更新日時（デフォルト）
    Updated(Order),
    /// 作成日時
    Created(Order),
    /// 記事番号
    Number(Order),
    /// Star の数
    Starts(Order),
    /// Watch の数
    Watchers(Order),
    /// Comment の数
    Comments(Order),
    /// 総合的な記事のスコア
    BestMatch(Order),
}

impl Into<(String, String)> for PostSort {
    fn into(self) -> (String, String) {
        match self {
            PostSort::Updated(ord) => (String::from("update"), ord.into()),
            PostSort::Created(ord) => (String::from("created"), ord.into()),
            PostSort::Number(ord) => (String::from("number"), ord.into()),
            PostSort::Starts(ord) => (String::from("stars"), ord.into()),
            PostSort::Watchers(ord) => (String::from("watchers"), ord.into()),
            PostSort::Comments(ord) => (String::from("comments"), ord.into()),
            PostSort::BestMatch(ord) => (String::from("best_match"), ord.into()),
        }
    }
}

pub enum Order {
    Desc,
    Asc,
}

impl From<String> for Order {
    fn from(s: String) -> Self {
        match s.as_str() {
            "asc" => Self::Asc,
            _ => Self::Desc,
        }
    }
}

impl Into<String> for Order {
    fn into(self) -> String {
        match self {
            Order::Asc => String::from("asc"),
            Order::Desc => String::from("desc"),
        }
    }
}
