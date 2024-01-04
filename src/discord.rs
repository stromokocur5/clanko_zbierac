use anyhow::{anyhow, Ok, Result};

#[derive(serde::Deserialize)]
pub struct Discord {
    pub token: String,
    pub channel_id: String,
}

pub fn html_to_markdown(content: &str) -> Result<String> {
    use scraper::{Html, Selector};
    let document = Html::parse_document(content);
    let mut markdown = String::new();

    let article_title = Selector::parse(r#"h1[data-don="article_title"]"#)
        .map_err(|_| anyhow!("no article_title"))?;

    let article_author = Selector::parse(r#"div[data-don="article_author"]"#)
        .map_err(|_| anyhow!("no article_author"))?;

    let article_perex = Selector::parse(r#"p[data-don="article_perex"]"#)
        .map_err(|_| anyhow!("no article_perex"))?;

    let datetime =
        Selector::parse(r#"span[class="datetime"]"#).map_err(|_| anyhow!("no daytime"))?;

    let article_body = Selector::parse(r#"div[data-don="article_body"]"#)
        .map_err(|_| anyhow!("no article_body"))?;

    Ok(document
        .select(&article_title)
        .next()
        .unwrap()
        .inner_html()
        .trim_start()
        .trim_end()
        .to_string())
}
