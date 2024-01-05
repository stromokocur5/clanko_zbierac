use clanko_zbierac::config_from_file;
use clanko_zbierac::medium::MediumClient;
use clanko_zbierac::trend::Trend;
use clanko_zbierac::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config_from_file()?;
    let client = MediumClient::new().await;
    let test = client.get_article(Trend::from(config),reqwest::Url::parse("https://trend.sk/biznis/tesla-nestiha-drzat-krok-tronu-mladom-trhu-coskoro-ujme-novy-lider-ciny?itm_brand=trend&itm_template=listing&itm_modul=articles-rubric-list&itm_position=6")?).await?;
    println!("{test}");
    Ok(())
}
