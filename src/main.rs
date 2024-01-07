use std::collections::HashMap;

use clanko_zbierac::medium::{Medium, MediumClient};
use clanko_zbierac::Result;
use clanko_zbierac::{config_from_file, trend::Trend};
use clap::{command, Parser};
use reqwest::Url;

/// Program na ziskanie clanku z medii do pdf suboru
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// link na clanok
    clanok: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = config_from_file()?;
    let mut media: HashMap<String, Box<dyn Medium>> = HashMap::new();
    media.insert("trend".to_string(), Box::new(Trend::from(config.clone())));
    let client = MediumClient::new(config).await;
    let (article, title) = client
        .get_article(&Url::parse(&args.clanok)?, &mut media)
        .await?;
    clanko_zbierac::markdown_to_pdf(&article, &title)?;
    Ok(())
}
