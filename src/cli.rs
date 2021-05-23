use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use anyhow::{bail, Context, Result};
use clap::Clap;
use dirs;

use crate::esa::{self, Esa};

#[derive(Clap, Debug)]
#[clap(
    name = "esa-cli",
    version = "0.0.0",
    author = "koizr",
    about = "esa.io as cli"
)]
struct Opts {
    #[clap(subcommand)]
    sub: SubCmd,
}

#[derive(Clap, Debug)]
enum SubCmd {
    /// Shows the team's information
    #[clap(name = "team")]
    Team,

    /// Shows or edits posts
    #[clap(name = "post")]
    Post {
        /// Post ID
        #[clap(name = "ID")]
        id: Option<i32>,

        /// Edits
        #[clap(short, long)]
        edit: bool,

        /// Views List
        #[clap(short, long)]
        list: bool,

        /// Create new post
        #[clap(short, long)]
        new: bool,
    },
}

pub async fn run() -> Result<()> {
    // コマンドイメージ
    // esa team
    // esa post 42 --get
    // esa post 42 --edit
    // esa post --list
    // esa post --list --team koizr

    let opts = Opts::parse();

    if let Some(path) = create_config_dir() {
        println!(
            "create config directory {}",
            path.as_os_str()
                .to_str()
                .expect("failed to get config directory path")
        );
    }
    if let Some(path) = create_tmp_file() {
        println!(
            "create edit temporarily file {}",
            path.as_os_str()
                .to_str()
                .expect("failed to get edit temporarily file path")
        );
    }

    let esa = Esa::new(
        esa::TeamId::new(
            env::var("ESA_TEAM_ID")
                .context("set your team ID to environment variable ESA_TEAM_ID.")?,
        ),
        esa::AccessToken::new(
            env::var("ESA_ACCESS_TOKEN")
                .context("set your access token to environment variable ESA_ACCESS_TOKEN.")?,
        ),
    );

    match opts.sub {
        SubCmd::Team => {
            let team = esa.team().await?;
            println!("team: {:?}", team);
        }
        SubCmd::Post {
            id,
            edit,
            list,
            new,
        } => {
            match id {
                Some(id) => {
                    if edit {
                        // TODO: 編集できるようにする
                        println!("Edit mode is not available yet.");
                    } else {
                        let post = esa.post(id).await?;
                        println!("{}", post.url);
                        println!("{}", post.full_name);
                        println!("{}", post.body_md);
                    }
                }
                None => {
                    if list {
                        // TODO: クエリを受け付ける
                        let posts = esa
                            .posts(esa::post::SearchQuery::new(None, None, None))
                            .await?;
                        for post in posts.posts {
                            println!("{}\t{}", post.number, post.name);
                        }
                    } else if new {
                        let exit_status = open_editor(&tmp_file_path(), TMP_FILE_DEFAULT_VALUE);
                        if exit_status.success() {
                            if let Some(diff) = get_diff() {
                                // TODO: esa.create_post を呼ぶ
                                println!("{}", diff);
                            } else {
                                println!("creating new post is canceled");
                            }
                        } else {
                            println!("creating new post is aborted");
                        }
                    } else {
                        bail!("Post ID argument or --list option are required.");
                    }
                }
            }
        }
    }

    Ok(())
}

/// return config directory path
fn config_dir_path() -> PathBuf {
    let mut config = dirs::home_dir().expect("home dir is not found");
    config.push(".esa");
    config
}

/// create config directory if it doesn't exist
fn create_config_dir() -> Option<PathBuf> {
    let config = config_dir_path();
    if config.exists() {
        None
    } else {
        fs::create_dir_all(&config).expect(&format!(
            "failed to config dir. path {}",
            config
                .as_os_str()
                .to_str()
                .expect("failed to get config directory path")
        ));
        Some(config)
    }
}

const TMP_FILE_DEFAULT_VALUE: &'static str = "default body";

fn tmp_file_path() -> PathBuf {
    let mut tmp_file_path = config_dir_path();
    tmp_file_path.push("edit.md");
    tmp_file_path
}

/// create temporarily file if it doesn't exist
fn create_tmp_file() -> Option<PathBuf> {
    let tmp_file_path = tmp_file_path();
    if tmp_file_path.exists() {
        None
    } else {
        File::create(&tmp_file_path).expect("failed to create new files");
        Some(tmp_file_path)
    }
}

fn editor() -> PathBuf {
    let editor = env::var("EDITOR").expect("EDITOR environment variable is not found");
    PathBuf::from(editor)
}

/// open text editor
fn open_editor(path: &PathBuf, default_text: &str) -> ExitStatus {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .expect("failed to open temporarily file")
        .write_all(default_text.as_bytes())
        .expect("failed to write to temporarily file");
    Command::new(editor())
        .arg(path)
        .spawn()
        .expect("failed to spawn text editor")
        .wait()
        .expect("failed to open editor")
}

fn read_tmp_file() -> String {
    fs::read_to_string(tmp_file_path()).expect("failed to read temporarily file")
}

/// if there is difference between default value and edited value, return edited value.
/// else return None
fn get_diff() -> Option<String> {
    let tmp_file_value = read_tmp_file();
    if &tmp_file_value[..] == TMP_FILE_DEFAULT_VALUE {
        None
    } else {
        Some(tmp_file_value)
    }
}
