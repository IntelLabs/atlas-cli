use crate::error::{Error, Result};
use crate::storage::traits::{ManifestMetadata, ManifestType, StorageBackend};
use atlas_c2pa_lib::claim::ClaimV2;
use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
use atlas_c2pa_lib::manifest::Manifest;
use std::collections::HashMap;
use std::sync::Mutex;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct MockStorageBackend {
    manifests: Mutex<HashMap<String, Manifest>>,
}

impl MockStorageBackend {
    pub fn new(manifest: Manifest) -> Self {
        let mut manifests = HashMap::new();
        manifests.insert(manifest.instance_id.clone(), manifest);
        Self {
            manifests: Mutex::new(manifests),
        }
    }

    pub fn add_manifest(&mut self, manifest: Manifest) {
        // Use lock() to get a mutable reference to the HashMap
        let mut manifests = self.manifests.lock().unwrap();
        manifests.insert(manifest.instance_id.clone(), manifest);
    }
    pub fn new_empty() -> Self {
        Self {
            manifests: Mutex::new(HashMap::new()),
        }
    }
}

impl StorageBackend for MockStorageBackend {
    fn get_base_uri(&self) -> String {
        "mock://".to_string()
    }

    fn store_manifest(&self, manifest: &Manifest) -> Result<String> {
        let mut manifests = self.manifests.lock().unwrap();
        let id = manifest.instance_id.clone();
        manifests.insert(id.clone(), manifest.clone());
        Ok(id)
    }

    fn retrieve_manifest(&self, id: &str) -> Result<Manifest> {
        let manifests = self.manifests.lock().unwrap();
        manifests
            .get(id)
            .cloned()
            .ok_or_else(|| Error::Storage(format!("Manifest not found: {id}")))
    }

    fn list_manifests(&self) -> Result<Vec<ManifestMetadata>> {
        let manifests = self.manifests.lock().unwrap();
        Ok(manifests
            .iter()
            .map(|(id, manifest)| ManifestMetadata {
                id: id.clone(),
                name: manifest.title.clone(),
                manifest_type: if manifest.ingredients.iter().any(|i| {
                    matches!(
                        i.data.data_types[0],
                        atlas_c2pa_lib::asset_type::AssetType::Dataset
                            | atlas_c2pa_lib::asset_type::AssetType::DatasetOnnx
                            | atlas_c2pa_lib::asset_type::AssetType::DatasetTensorFlow
                            | atlas_c2pa_lib::asset_type::AssetType::DatasetPytorch
                    )
                }) {
                    ManifestType::Dataset
                } else {
                    ManifestType::Model
                },
                created_at: manifest.created_at.0.to_string(),
            })
            .collect())
    }

    fn delete_manifest(&self, id: &str) -> Result<()> {
        let mut manifests = self.manifests.lock().unwrap();
        manifests
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| Error::Storage(format!("Manifest not found: {id}")))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn create_default_claim() -> ClaimV2 {
    ClaimV2 {
        instance_id: format!("urn:c2pa:{}", Uuid::new_v4()),
        ingredients: vec![],
        created_assertions: vec![],
        claim_generator_info: "test".to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    }
}
