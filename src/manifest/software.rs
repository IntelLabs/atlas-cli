use crate::error::Result;
use crate::manifest::common::{list_manifests, verify_manifest, AssetKind};
use crate::manifest::config::ManifestCreationConfig;
use crate::storage::traits::StorageBackend;

pub fn create_manifest(
    mut config: ManifestCreationConfig,
    software_type: String,
    version: Option<String>,
) -> Result<()> {
    config.software_type = Some(software_type.clone());
    config.version = version.clone();

    // Combine software_type and version into description or metadata
    let enhanced_description = match (&config.description, &version) {
        (Some(desc), Some(ver)) => Some(format!(
            "{} (Type: {}, Version: {})",
            desc, software_type, ver
        )),
        (Some(desc), None) => Some(format!("{} (Type: {})", desc, software_type)),
        (None, Some(ver)) => Some(format!("Type: {}, Version: {}", software_type, ver)),
        (None, None) => Some(format!("Type: {}", software_type)),
    };

    // Update the description in the config
    config.description = enhanced_description;

    // Call the common implementation with AssetKind::Software
    crate::manifest::common::create_manifest(config, AssetKind::Software)
}

/// List software manifests
pub fn list_software_manifests(storage: &dyn StorageBackend) -> Result<()> {
    // Call the unified implementation with AssetKind::Software
    list_manifests(storage, Some(AssetKind::Software))
}

/// Verify a software manifest
pub fn verify_software_manifest(id: &str, storage: &dyn StorageBackend) -> Result<()> {
    // Call the unified implementation
    verify_manifest(id, storage)
}
