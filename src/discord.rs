#[derive(serde::Deserialize)]
pub struct Discord {
    pub token: String,
    pub channel_id: String,
}
