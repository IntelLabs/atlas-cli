pub mod config;
pub mod database;
pub mod filesystem;
pub mod rekor;
pub mod traits;
use crate::error::Result;
pub use database::DatabaseStorage;
pub use filesystem::FilesystemStorage;
pub use rekor::RekorStorage;
pub use traits::{ManifestMetadata, ManifestType, StorageBackend};

pub fn initialize_storage() -> Result<RekorStorage> {
    RekorStorage::new()
}

pub fn create_storage(storage_type: &str, url: String) -> Result<Box<dyn StorageBackend>> {
    match storage_type {
        "database" => Ok(Box::new(DatabaseStorage::new(url)?)),
        "rekor" => Ok(Box::new(RekorStorage::new_with_url(url)?)),
        "local-fs" => Ok(Box::new(FilesystemStorage::new(url)?)),
        // Backwards compatibility with warnings
        "local" => {
            eprintln!("Warning: Storage type 'local' is deprecated and will be removed in a future version. Use 'database' instead.");
            Ok(Box::new(DatabaseStorage::new(url)?))
        }
        "filesystem" => {
            eprintln!("Warning: Storage type 'filesystem' is deprecated and will be removed in a future version. Use 'local-fs' instead.");
            Ok(Box::new(FilesystemStorage::new(url)?))
        }
        _ => Err(crate::error::Error::Validation(
            "Invalid storage type. Valid options are: database, rekor, local-fs".to_string(),
        )),
    }
}
