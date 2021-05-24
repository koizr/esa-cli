use anyhow::{anyhow, Result};

use crate::esa;

pub const TMP_FILE_DEFAULT_VALUE: &'static str = r#"<!-- ### input post name next line ### -->

<!-- ### input body next and subsequent lines ### -->
"#;

pub fn parse_new_post(content: &str) -> Result<esa::post::NewPost> {
    let mut lines = content.lines();
    if let None = lines.next() {
        Err(anyhow!("failed to parse content"))?
    }
    let title = lines.next();
    let ParsedTitle {
        category,
        name,
        tags,
    } = match title {
        Some(title) => parse_title(title)?,
        None => Err(anyhow!("failed to parse content. post name is required"))?,
    };
    if let None = lines.next() {
        Err(anyhow!("failed to parse content"))?
    }
    let mut body = Vec::new();
    for line in lines {
        body.push(line);
    }

    Ok(esa::post::NewPost {
        name,
        body_md: Some(body.join("\n")),
        tags,
        category,
        wip: true,
        message: None,
    })
}

struct ParsedTitle {
    category: Option<String>,
    name: String,
    tags: Vec<String>,
}

fn parse_title(source: &str) -> Result<ParsedTitle> {
    let (category, name, tags) = {
        if source.len() < 1 {
            Err(anyhow!("failed to parse content. post name is required"))?
        };
        let title = source.split("/").collect::<Vec<_>>();
        let category = title[..title.len() - 1].join("/");
        let (name, tags) = parse_name_and_tags(title[title.len() - 1]);
        (
            if category.len() == 0 {
                None
            } else {
                Some(String::from(category))
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
            esa::post::NewPost {
                name: String::from("記事タイトル"),
                body_md: Some(String::from("記事 body")),
                category: Some(String::from("カテゴリ1/カテゴリ2")),
                tags: vec![String::from("tag1"), String::from("タグ2")],
                wip: true,
                message: None,
            },
            parse_new_post(
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
