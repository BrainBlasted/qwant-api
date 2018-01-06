#[derive(Deserialize)]
pub struct Media {
    pub url: String,
    pub width: u64,
    pub height: u64,
    pub type_: String,
}
