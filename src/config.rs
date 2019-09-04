#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub language: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: "en_GB".to_owned(),
        }
    }
}
