use std::env;

use anyhow::{bail, Context, Result};
use clap::Clap;

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
            let team = esa.team().await;
            match team {
                Ok(team) => println!("team: {:?}", team),
                Err(reason) => println!("error: {}", reason),
            }
        }
        SubCmd::Post { id, edit, list } => {
            match id {
                Some(id) => {
                    if edit {
                        // TODO: 編集できるようにする
                        println!("Edit mode is not available yet.");
                    } else {
                        // TODO: unwrap を使わないようにするために esa でも anyhow を使う
                        let post = esa.post(id).await.unwrap();
                        println!("{}", post.url);
                        println!("{}", post.full_name);
                        println!("{}", post.body_md);
                    }
                }
                None => {
                    if list {
                        // TODO: クエリを受け付ける
                        // TODO: unwrap を使わないようにするために esa でも anyhow を使う
                        let posts = esa
                            .posts(esa::post::SearchQuery::new(None, None, None))
                            .await
                            .unwrap();
                        for post in posts.posts {
                            println!("{}\t{}", post.number, post.name);
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
