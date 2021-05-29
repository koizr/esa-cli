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

        /// Creates new post
        #[clap(short, long)]
        new: bool,

        /// Deletes the post
        #[clap(short, long)]
        delete: bool,

        /// Views posts
        #[clap(short, long)]
        list: bool,

        /// Filters posts
        #[clap(short, long)]
        query: Option<String>,

        /// Includes optional data
        #[clap(short, long)]
        include: Option<Vec<String>>,

        /// Sorts posts
        #[clap(short, long)]
        sort: Option<String>,

        /// Sorting order
        #[clap(short, long)]
        order: Option<String>,
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
            print_team(&esa).await?;
        }
        SubCmd::Post {
            id,
            edit,
            new,
            delete,
            list,
            query,
            include,
            sort,
            order,
        } => match id {
            Some(id) => {
                if edit {
                    edit_post(&esa, id, &config).await?;
                } else if delete {
                    delete_post(&esa, id).await?;
                } else {
                    print_post(&esa, id).await?;
                }
            }
            None => {
                if list {
                    print_posts(&esa, query, include, sort, order).await?;
                } else if new {
                    create_post(&esa, &config).await?;
                } else {
                    bail!("Post ID argument or --list option are required.");
                }
            }
        },
    }

    Ok(())
}

/// Print team
async fn print_team(esa: &Esa) -> Result<()> {
    let team = esa.team().await?;
    println!("team: {:?}", team);
    Ok(())
}

/// Print post
/// # Args
/// - id: Post ID
async fn print_post(esa: &Esa, id: i32) -> Result<()> {
    let post = esa.post(id).await?;
    println!("{}", post.url);
    println!("{}", post.full_name);
    println!("{}", post.body_md);
    Ok(())
}

/// Print posts
async fn print_posts(
    esa: &Esa,
    query: Option<String>,
    include: Option<Vec<String>>,
    sort: Option<String>,
    order: Option<String>,
) -> Result<()> {
    let include: Option<Vec<esa::post::Include>> =
        include.map(|include| include.into_iter().map(|i| i.into()).collect());
    let sort = sort.map(|s| esa::post::Sort::from((s, order)));
    let search_query = esa::post::SearchQuery::new(query, include, sort);
    log::debug!("{:?}", &search_query);

    let posts = esa.posts(search_query).await?;
    for post in posts.posts {
        println!("{}\t{}", post.number, post.full_name);
    }
    Ok(())
}

/// Create new post
async fn create_post(esa: &Esa, config: &Config) -> Result<()> {
    let editor = Editor::new(config);
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
    Ok(())
}

/// Edit post
/// # Args
/// - id: Post ID
async fn edit_post(esa: &Esa, id: i32, config: &Config) -> Result<()> {
    let post = esa.post(id).await?;
    let post_content = tmp_file::format_post_content(&post.full_name, &post.body_md);

    let editor = Editor::new(config);
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
    Ok(())
}

/// Delete post
/// # Args
/// - id: Post ID
async fn delete_post(esa: &Esa, id: i32) -> Result<()> {
    let post = esa.post(id).await?;
    println!("post {}: {}", id, post.full_name);
    if confirm("Do you delete the above post")? {
        esa.delete_post(id).await?;
        println!("{} is deleted.", post.full_name);
    } else {
        println!("canceled");
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
