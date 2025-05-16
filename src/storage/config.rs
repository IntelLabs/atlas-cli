#[derive(Debug, Clone)]
pub enum StorageType {
    Rekor,
    Database,
    LocalFs,
}
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub rekor_url: String,
    pub enable_verification: bool,
    pub filesystem_path: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            rekor_url: "https://rekor.sigstore.dev".to_string(),
            enable_verification: true,
            filesystem_path: None,
        }
    }
}
