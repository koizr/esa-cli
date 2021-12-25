use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, Result};

use super::config::Env;

pub const TMP_FILE_DEFAULT_VALUE: &str = r#"<!-- ### input post name next line ### -->

<!-- ### input body next and subsequent lines ### -->
"#;

pub fn format_post_content(title: &str, body: &str) -> String {
    format!(
        r#"<!-- ### input post name next line ### -->
{}
<!-- ### input body next and subsequent lines ### -->
{}
"#,
        title, body
    )
}

pub struct Editor<'a> {
    env: &'a Env,
}

impl<'a> Editor<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    pub fn open(&self, default_text: &str) -> ExitStatus {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.env.tmp_file_path)
            .expect("failed to open temporarily file")
            .write_all(default_text.as_bytes())
            .expect("failed to write to temporarily file");

        log::debug!(
            "open editor {}",
            self.env
                .editor_path
                .as_os_str()
                .to_str()
                .unwrap_or("<no editor path>")
        );

        let status = Command::new(&self.env.editor_path)
            .arg(&self.env.tmp_file_path)
            .spawn()
            .expect("failed to spawn text editor")
            .wait()
            .expect("failed to open editor");

        log::debug!(
            "close editor with exit status {}",
            status
                .code()
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("<no exit status>"))
        );

        status
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.env.tmp_file_path).expect("failed to read temporarily file")
    }

    pub fn diff(&self) -> Option<String> {
        let tmp_file_value = self.read();
        if &tmp_file_value[..] == TMP_FILE_DEFAULT_VALUE {
            None
        } else {
            Some(tmp_file_value)
        }
    }
}

pub fn parse_post(content: &str) -> Result<esa::post::PostContent> {
    let mut lines = content.lines();
    if lines.next().is_none() {
        return Err(anyhow!("failed to parse content"));
    }
    let title = lines
        .next()
        .ok_or(anyhow!("failed to parse content. post name is required"))?;
    let ParsedTitle {
        category,
        name,
        tags,
    } = parse_title(title)?;
    if lines.next().is_none() {
        return Err(anyhow!("failed to parse content"));
    }
    let mut body = Vec::new();
    for line in lines {
        body.push(line);
    }

    Ok(esa::post::PostContent {
        name,
        full_name: String::from(title),
        body_md: Some(body.join("\n")),
        tags,
        category,
    })
}

struct ParsedTitle {
    category: Option<String>,
    name: String,
    tags: Vec<String>,
}

fn parse_title(source: &str) -> Result<ParsedTitle> {
    let (category, name, tags) = {
        if source.is_empty() {
            return Err(anyhow!("failed to parse content. post name is required"));
        };
        let title = source.split('/').collect::<Vec<_>>();
        let category = title[..title.len() - 1].join("/");
        let (name, tags) = parse_name_and_tags(title[title.len() - 1]);
        (
            if category.is_empty() {
                None
            } else {
                Some(category)
            },
            name,
            tags,
        )
    };
    Ok(ParsedTitle {
        category,
        name,
        tags,
    })
}

fn parse_name_and_tags(source: &str) -> (String, Vec<String>) {
    let parts = source.split(" #").collect::<Vec<_>>();
    let name = String::from(parts[0]);
    let tags = if parts.len() >= 2 {
        let tags = &parts[1..];
        tags.iter().map(|&s| String::from(s)).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    (name, tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_new_post() {
        assert_eq!(
            esa::post::PostContent {
                name: String::from("記事タイトル"),
                full_name: String::from("カテゴリ1/カテゴリ2/記事タイトル #tag1 #タグ2"),
                body_md: Some(String::from("記事 body")),
                category: Some(String::from("カテゴリ1/カテゴリ2")),
                tags: vec![String::from("tag1"), String::from("タグ2")],
            },
            parse_post(
                r#"<!-- ### input post name next line ### -->
カテゴリ1/カテゴリ2/記事タイトル #tag1 #タグ2
<!-- ### input body next and subsequent lines ### -->
記事 body
"#
            )
            .unwrap()
        )
    }

    #[test]
    fn test_parse_name_and_tags() {
        assert_eq!(
            (
                String::from("記事タイトル"),
                vec![String::from("tag1"), String::from("タグ2")]
            ),
            parse_name_and_tags("記事タイトル #tag1 #タグ2")
        )
    }
}
