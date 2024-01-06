use clanko_zbierac::config_from_file;
use clanko_zbierac::medium::MediumClient;
use clanko_zbierac::Result;
use clap::{command, Parser};
use reqwest::Url;

/// Program na ziskanie clanku z medii do pdf suboru
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// link na clanok
    #[arg(short, long)]
    clanok: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = config_from_file()?;
    let client = MediumClient::new(config).await;
    let (article, title) = client.get_article(&Url::parse(&args.clanok)?).await?;
    clanko_zbierac::markdown_to_pdf(&article, &title)?;
    Ok(())
}
