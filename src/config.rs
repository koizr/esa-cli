use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use dirs;
use serde::Deserialize;
use serde_json::from_reader;

use crate::esa::{Team, TeamId};

#[derive(Debug)]
pub struct Env {
    pub dir_path: PathBuf,
    pub config_file_path: PathBuf,
    pub tmp_file_path: PathBuf,
    pub editor_path: PathBuf,
}

impl Env {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        let dir_path = config_dir.unwrap_or_else(|| {
            let mut home = dirs::home_dir().expect("home dir is not found");
            home.push(".esa");
            home
        });

        let config_file_path = {
            let mut config = dir_path.clone();
            config.push("config.json");
            config
        };

        let tmp_file_path = {
            let mut config = dir_path.clone();
            config.push("edit.md");
            config
        };

        let editor_path = {
            let editor = env::var("EDITOR").expect("EDITOR environment variable is not found");
            PathBuf::from(editor)
        };

        let esa_env = Self {
            dir_path,
            config_file_path,
            tmp_file_path,
            editor_path,
        };
        esa_env.init();
        esa_env
    }

    fn init(&self) {
        // config dir
        if !self.dir_path.exists() {
            fs::create_dir_all(&self.dir_path).expect(&format!(
                "failed to config dir. path {}",
                self.dir_path
                    .as_os_str()
                    .to_str()
                    .expect("failed to get config directory path")
            ));
        }

        // config file
        if !self.config_file_path.exists() {
            let mut file = File::create(&self.tmp_file_path).expect("failed to create config file");
            // JSON パースエラーにならないように空オブジェクトを入れておく
            file.write_all("{}".as_bytes())
                .expect("failed to initialize config file");
        }

        // tmp file
        if !self.tmp_file_path.exists() {
            File::create(&self.tmp_file_path).expect("failed to create tmp file");
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    default_team: Option<Team>,
    teams: Option<Vec<Team>>,
}

impl Config {
    pub fn new(env: &Env) -> Self {
        let config_file = File::open(&env.config_file_path).expect("failed to read config file");
        let reader = BufReader::new(config_file);
        from_reader(reader).expect("invalid config file format")
    }

    pub fn get(&self, team_id: TeamId) -> Option<&Team> {
        match self.default_team {
            Some(ref team) if team.id == team_id => Some(&team),
            _ => match self.teams {
                Some(ref teams) => teams.iter().find(|team| team.id == team_id),
                None => None,
            },
        }
    }

    pub fn default(&self) -> Option<&Team> {
        self.default_team.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    use crate::esa::{AccessToken, Team, TeamId};

    #[test]
    fn test_parse_config() {
        assert_eq!(
            from_str::<Config>(
                r#"
                {
                    "default_team": {
                        "id": "test_team1",
                        "access_token": "test_access_token1"
                    },
                    "teams": [
                        {
                            "id": "test_team2",
                            "access_token": "test_access_token2"
                        },
                        {
                            "id": "test_team3",
                            "access_token": "test_access_token3"
                        }
                    ]
                }
                "#
            )
            .unwrap(),
            Config {
                default_team: Some(Team {
                    id: TeamId::new(String::from("test_team1")),
                    access_token: AccessToken::new(String::from("test_access_token1")),
                }),
                teams: Some(vec![
                    Team {
                        id: TeamId::new(String::from("test_team2")),
                        access_token: AccessToken::new(String::from("test_access_token2")),
                    },
                    Team {
                        id: TeamId::new(String::from("test_team3")),
                        access_token: AccessToken::new(String::from("test_access_token3")),
                    },
                ])
            }
        );
    }
}
