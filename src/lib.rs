use anyhow::Result;

pub mod discord;
pub mod trend;

#[derive(serde::Deserialize)]
pub struct Config {
    pub trend: trend::Trend,
    pub discord: discord::Discord,
}

pub fn config_from_file() -> Result<Config> {
    let config = std::fs::read_to_string("config.toml").expect("ziaden subor config.toml");
    let config: Config = toml::from_str(&config)?;
    Ok(config)
}

pub fn markdown_to_pdf(content: &str) -> Result<()> {
    todo!()
}
