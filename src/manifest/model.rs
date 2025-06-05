use crate::error::Result;
use crate::manifest::common::{AssetKind, list_manifests, verify_manifest};
use crate::manifest::config::ManifestCreationConfig;
use crate::storage::traits::StorageBackend;

pub fn create_manifest(config: ManifestCreationConfig) -> Result<()> {
    crate::manifest::common::create_manifest(config, AssetKind::Model)
}

/// List model manifests
pub fn list_model_manifests(storage: &dyn StorageBackend) -> Result<()> {
    // Call the unified implementation with AssetKind::Model
    list_manifests(storage, Some(AssetKind::Model))
}

/// Verify a model manifest
pub fn verify_model_manifest(id: &str, storage: &dyn StorageBackend) -> Result<()> {
    // Call the unified implementation
    verify_manifest(id, storage)
}
