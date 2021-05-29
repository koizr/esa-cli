use std::env;
use std::io;

use anyhow::{bail, Context, Result};
use clap::Clap;
use log;

use crate::esa::{self, Esa};

mod config;
mod tmp_file;

use config::Config;
use tmp_file::Editor;

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

        /// Edits the post
        #[clap(short, long)]
        edit: bool,

        /// Views posts
        #[clap(short, long)]
        list: bool,

        /// Creates new post
        #[clap(short, long)]
        new: bool,

        /// Deletes the post
        #[clap(short, long)]
        delete: bool,
    },
}

pub async fn run() -> Result<()> {
    let opts = Opts::parse();
    log::debug!("Options: {:?}", opts);
    let config = Config::new();
    log::debug!("Config: {:?}", config);

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
            delete,
        } => {
            match id {
                Some(id) => {
                    if edit {
                        let post = esa.post(id).await?;
                        let post_content =
                            tmp_file::format_post_content(&post.full_name, &post.body_md);

                        let editor = Editor::new(&config);
                        let exit_status = editor.open(&post_content);

                        if exit_status.success() {
                            if let Some(diff) = editor.diff() {
                                let post_content = tmp_file::parse_post(&diff)?;
                                let edited_post = post.edit(
                                    post_content.full_name,
                                    post_content.body_md,
                                    Some(post_content.tags),
                                    post_content.category,
                                    true,
                                    None,
                                );
                                let edited = esa.edit_post(id, &edited_post).await?;
                                println!("Edit post! {}", edited.url);
                            } else {
                                println!("creating new post is canceled");
                            }
                        } else {
                            println!("creating new post is aborted");
                        }
                    } else if delete {
                        let post = esa.post(id).await?;
                        println!("post {}: {}", id, post.full_name);
                        if confirm("Do you delete the above post")? {
                            esa.delete_post(id).await?;
                            println!("{} is deleted.", post.full_name);
                        } else {
                            println!("canceled");
                        }
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
                            println!("{}\t{}", post.number, post.full_name);
                        }
                    } else if new {
                        let editor = Editor::new(&config);
                        let exit_status = editor.open(tmp_file::TMP_FILE_DEFAULT_VALUE);
                        if exit_status.success() {
                            if let Some(diff) = editor.diff() {
                                let post_content = tmp_file::parse_post(&diff)?;
                                let created = esa.create_post(post_content, true, None).await?;
                                println!("Create new post! {}", created.url);
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

/// print confirm message
/// # Returns
/// - Ok(true): input yes
/// - Ok(false): input others
/// - Err: input error
fn confirm(message: &str) -> Result<bool> {
    // TODO: (y/N): _ ← ここに入力できるようにしたい（改行をはさみたくない）
    println!("{} (y/N): ", message);

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    match &(answer.trim().to_lowercase())[..] {
        "y" | "yes" => Ok(true),
        _ => Ok(false),
    }
}
