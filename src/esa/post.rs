use chrono::{prelude::Local, DateTime};
use serde::{Deserialize, Serialize};

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
    pub category: Option<String>,
    pub revision_number: i32,
    pub created_by: Writer,
    pub updated_by: Writer,
    pub kind: Kind,
    pub comments_count: i32,
    pub tasks_count: i32,
    pub done_tasks_count: i32,
    pub stargazers_count: i32,
    pub watchers_count: i32,
    pub star: bool,
    pub watch: bool,
    // TODO: comments, stargazers を追加する
}

impl Post {
    pub fn edit(
        self,
        name: String,
        body_md: Option<String>,
        tags: Option<Vec<String>>,
        category: Option<String>,
        wip: bool,
        message: Option<String>,
    ) -> EditedPost {
        let Self {
            body_md: original_body_md,
            revision_number: original_number,
            updated_by:
                Writer {
                    screen_name: original_user,
                    ..
                },
            ..
        } = self;
        EditedPost {
            name,
            body_md,
            tags,
            category,
            wip,
            message,
            original_revision: Some(OriginalRevisionPost {
                body_md: Some(original_body_md),
                number: Some(original_number),
                user: Some(original_user),
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Writer {
    pub myself: bool,
    pub name: String,
    pub screen_name: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub enum Kind {
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

#[derive(Debug)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub include: Option<Vec<Include>>,
    pub sort: Option<Sort>,
}

impl SearchQuery {
    pub fn new(q: Option<String>, include: Option<Vec<Include>>, sort: Option<Sort>) -> Self {
        SearchQuery { q, include, sort }
    }
}

#[derive(Debug)]
pub enum Include {
    /// スターを含む
    Stargazers,
    /// コメントを含む
    Comments,
    /// コメントに対するスターを含む
    CommentStargazers,
    /// なし
    None,
}

impl From<String> for Include {
    fn from(s: String) -> Self {
        match s.as_str() {
            "stargazers" => Self::Stargazers,
            "comments" => Self::Comments,
            "comments.stargazers" => Self::CommentStargazers,
            _ => Self::None,
        }
    }
}

impl Into<String> for Include {
    fn into(self) -> String {
        match self {
            Self::Stargazers => "stargazers".to_string(),
            Self::Comments => "comments".to_string(),
            Self::CommentStargazers => "comments.stargazers".to_string(),
            Self::None => "".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Sort {
    /// 更新日時（デフォルト）
    Updated(Order),
    /// 作成日時
    Created(Order),
    /// 記事番号
    Number(Order),
    /// Star の数
    Stars(Order),
    /// Watch の数
    Watchers(Order),
    /// Comment の数
    Comments(Order),
    /// 総合的な記事のスコア
    BestMatch(Order),
}

impl From<(String, String)> for Sort {
    fn from(value: (String, String)) -> Self {
        let (sort, order) = value;
        match &sort[..] {
            "created" => Self::Created(From::from(order)),
            "number" => Self::Number(From::from(order)),
            "stars" => Self::Stars(From::from(order)),
            "watchers" => Self::Watchers(From::from(order)),
            "comments" => Self::Comments(From::from(order)),
            "best_match" => Self::BestMatch(From::from(order)),
            _ => Self::Updated(From::from(order)),
        }
    }
}

impl From<(String, Option<String>)> for Sort {
    fn from(value: (String, Option<String>)) -> Self {
        let order = value.1.unwrap_or_default();
        Self::from((value.0, order))
    }
}

impl Into<(String, String)> for Sort {
    fn into(self) -> (String, String) {
        match self {
            Sort::Updated(ord) => (String::from("update"), ord.into()),
            Sort::Created(ord) => (String::from("created"), ord.into()),
            Sort::Number(ord) => (String::from("number"), ord.into()),
            Sort::Stars(ord) => (String::from("stars"), ord.into()),
            Sort::Watchers(ord) => (String::from("watchers"), ord.into()),
            Sort::Comments(ord) => (String::from("comments"), ord.into()),
            Sort::BestMatch(ord) => (String::from("best_match"), ord.into()),
        }
    }
}

#[derive(Debug)]
pub enum Order {
    Desc,
    Asc,
}

impl Default for Order {
    fn default() -> Self {
        Order::Desc
    }
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

#[derive(Debug, Serialize, PartialEq)]
pub struct PostContent {
    pub name: String,
    pub full_name: String,
    pub body_md: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct NewPost {
    pub name: String,
    pub body_md: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub wip: bool,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewPostCreated {
    pub number: i32,
    pub name: String,
    pub full_name: String,
    pub wip: bool,
    pub body_md: Option<String>,
    pub body_html: Option<String>,
    pub created_at: DateTime<Local>,
    pub message: Option<String>,
    pub url: String,
    pub updated_at: DateTime<Local>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub revision_number: i32,
    pub created_by: Writer,
    pub updated_by: Writer,
    pub kind: Kind,
    pub comments_count: i32,
    pub tasks_count: i32,
    pub done_tasks_count: i32,
    pub stargazers_count: i32,
    pub watchers_count: i32,
    pub star: bool,
    pub watch: bool,
}

#[derive(Debug, Serialize)]
pub struct EditedPost {
    pub name: String,
    pub body_md: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub wip: bool,
    pub message: Option<String>,
    pub original_revision: Option<OriginalRevisionPost>,
}

#[derive(Debug, Serialize)]
pub struct OriginalRevisionPost {
    pub body_md: Option<String>,
    pub number: Option<i32>,
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostEdited {
    pub number: i32,
    pub name: String,
    pub full_name: String,
    pub wip: bool,
    pub body_md: Option<String>,
    pub body_html: Option<String>,
    pub created_at: DateTime<Local>,
    pub message: Option<String>,
    pub url: String,
    pub updated_at: DateTime<Local>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub revision_number: i32,
    pub created_by: Writer,
    pub updated_by: Writer,
    pub overlapped: bool,
    pub kind: Kind,
    pub comments_count: i32,
    pub tasks_count: i32,
    pub done_tasks_count: i32,
    pub stargazers_count: i32,
    pub watchers_count: i32,
    pub star: bool,
    pub watch: bool,
}
