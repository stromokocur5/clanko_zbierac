pub use anyhow::Result;

pub mod medium;

pub mod aktuality;
pub mod dennikn;
pub mod idnes;
pub mod sme;
pub mod trend;

#[derive(serde::Deserialize, Clone)]
pub struct MediaConfig {
    pub trend: Option<trend::Trend>,
    pub sme: Option<sme::Sme>,
    pub dennikn: Option<dennikn::DennikN>,
    pub aktuality: Option<aktuality::Aktuality>,
    pub idnes: Option<idnes::Idnes>,
}

pub fn config_from_file() -> Result<MediaConfig> {
    let config = std::fs::read_to_string("config.toml").unwrap_or_default();
    let config: MediaConfig = toml::from_str(&config).unwrap_or_else(|_| MediaConfig {
        trend: None,
        sme: None,
        dennikn: None,
        aktuality: None,
        idnes: None,
    });
    Ok(config)
}

pub fn markdown_to_pdf(content: &str, name: &str) -> Result<()> {
    let md = format!("{name}.md").to_string();
    let pdf = format!("{name}.pdf");
    std::fs::write(&md, &content)?;
    let mut pandoc = pandoc::new();
    pandoc.add_input(&md);
    pandoc.set_output(pandoc::OutputKind::File(pdf.into()));
    pandoc.execute()?;
    std::fs::remove_file(md)?;
    Ok(())
}
