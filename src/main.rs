use anyhow::{anyhow, Result};
use trendbot::config_from_file;
use trendbot::{discord, trend};

#[tokio::main]
async fn main() -> Result<()> {
    let config = config_from_file()?;
    let client = trend::get_client(config).await?;
    let test = trend::get_article(&client,reqwest::Url::parse("https://trend.sk/biznis/tesla-nestiha-drzat-krok-tronu-mladom-trhu-coskoro-ujme-novy-lider-ciny?itm_brand=trend&itm_template=listing&itm_modul=articles-rubric-list&itm_position=6")?).await?;
    println!("{test}");

    Ok(())
}
