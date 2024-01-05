use anyhow::Result;

pub mod aktuality;
pub mod dennikn;
pub mod idnes;
pub mod sme;
pub mod trend;

#[derive(serde::Deserialize)]
pub struct MediaConfig {
    pub trend: trend::Trend,
    pub sme: sme::Sme,
    pub dennikn: dennikn::DennikN,
    pub aktuality: aktuality::Aktuality,
    pub idnes: idnes::Idnes,
}

pub fn config_from_file() -> Result<MediaConfig> {
    let config = std::fs::read_to_string("config.toml").expect("ziaden subor config.toml");
    let config: MediaConfig = toml::from_str(&config)?;
    Ok(config)
}

pub fn markdown_to_pdf(content: &str) -> Result<()> {
    todo!()
}
