use serde::Deserialize;

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
