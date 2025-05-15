use crate::error::Result;
use crate::manifest::common::{list_manifests, verify_manifest, AssetKind};
use crate::manifest::config::ManifestCreationConfig;
use crate::storage::traits::StorageBackend;
use c2pa_ml::asset_type::AssetType;
use c2pa_ml::ingredient::{Ingredient, IngredientData};
use std::path::Path;
use uuid::Uuid;

pub fn create_manifest(config: ManifestCreationConfig) -> Result<()> {
    crate::manifest::common::create_manifest(config, AssetKind::Dataset)
}

/// List dataset manifests
pub fn list_dataset_manifests(storage: &dyn StorageBackend) -> Result<()> {
    // Call the unified implementation with AssetKind::Dataset
    list_manifests(storage, Some(AssetKind::Dataset))
}

/// Verify a dataset manifest
pub fn verify_dataset_manifest(id: &str, storage: &dyn StorageBackend) -> Result<()> {
    // Call the unified implementation
    verify_manifest(id, storage)
}

#[allow(dead_code)]
fn create_ingredient_from_path(
    path: &Path,
    name: &str,
    asset_type: AssetType,
    format: String,
) -> Result<Ingredient> {
    let ingredient_data = IngredientData {
        url: path.to_string_lossy().to_string(),
        alg: "sha256".to_string(),
        hash: crate::hash::calculate_file_hash(path)?,
        data_types: vec![asset_type],
        linked_ingredient_url: None,
        linked_ingredient_hash: None,
    };

    Ok(Ingredient {
        title: name.to_string(),
        format,
        relationship: "componentOf".to_string(),
        document_id: format!("uuid:{}", Uuid::new_v4()),
        instance_id: format!("uuid:{}", Uuid::new_v4()),
        data: ingredient_data,
        linked_ingredient: None,
        public_key: None,
    })
}
