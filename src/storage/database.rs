use crate::error::{Error, Result};
use crate::manifest::utils::{determine_manifest_type, manifest_type_to_string};
use crate::storage::traits::{ManifestMetadata, ManifestType, StorageBackend};
use atlas_c2pa_lib::manifest::Manifest;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseStorage {
    base_url: String,
    client: Client,
}

impl DatabaseStorage {
    pub fn print_manifest_structure(value: &Value, indent: usize) {
        let spaces = " ".repeat(indent);
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    println!("{spaces}{key}: ");
                    Self::print_manifest_structure(value, indent + 2);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    Self::print_manifest_structure(value, indent + 2);
                }
            }
            _ => println!("{spaces}{value}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<serde_json::Value>,
    manifest_id: String,
    manifest_type: String,
    manifest: Value,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestWrapper {
    manifest_id: String,
    manifest_type: String,
    #[serde(flatten)]
    manifest: Manifest,
    created_at: String,
}

impl DatabaseStorage {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Storage(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            base_url: url.trim_end_matches('/').to_string(),
            client,
        })
    }

    fn manifest_url(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("{}/manifests/{}", self.base_url, id),
            None => format!("{}/manifests", self.base_url),
        }
    }
}

impl StorageBackend for DatabaseStorage {
    fn get_base_uri(&self) -> String {
        self.base_url.clone()
    }

    fn store_manifest(&self, manifest: &Manifest) -> Result<String> {
        // Check if this ID already exists
        let existing = self
            .client
            .get(format!(
                "{}/manifests/{}",
                self.base_url, &manifest.instance_id
            ))
            .send()
            .map_err(|e| Error::Storage(format!("Failed to check existing manifest: {e}")))?;

        if existing.status().is_success() {
            // Manifest exists - create a new version

            // Parse the existing ID
            let parts: Vec<&str> = manifest.instance_id.split(':').collect();
            let uuid_part = if parts.len() >= 3 {
                parts[2].to_string() // Extract UUID from urn:c2pa:UUID format
            } else {
                manifest.instance_id.clone() // Use as-is if not in expected format
            };

            // Extract claim generator info
            let claim_generator = manifest.claim_generator.replace('/', "_");

            // Get all manifests to find highest version
            let all_manifests_response = self
                .client
                .get(format!("{}/manifests", self.base_url))
                .send()
                .map_err(|e| Error::Storage(format!("Failed to list manifests: {e}")))?;

            let all_manifests: Vec<serde_json::Value> = all_manifests_response
                .json()
                .map_err(|e| Error::Storage(format!("Failed to parse manifests list: {e}")))?;

            // Find highest version for this ID
            let mut max_version = 0;
            for manifest_entry in all_manifests {
                if let Some(id) = manifest_entry.get("manifest_id").and_then(|v| v.as_str()) {
                    if id.starts_with(&format!("urn:c2pa:{uuid_part}:")) {
                        let id_parts: Vec<&str> = id.split(':').collect();
                        if id_parts.len() >= 5 {
                            if let Some(version_reason) = id_parts.get(4) {
                                if let Some(version_str) = version_reason.split('_').next() {
                                    if let Ok(version) = version_str.parse::<i32>() {
                                        max_version = max_version.max(version);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Create new versioned ID
            // Reason code 1 = Updated manifest
            let versioned_id = format!(
                "urn:c2pa:{}:{}:{}_{}",
                uuid_part,
                claim_generator,
                max_version + 1,
                1
            );

            // Create a copy of the manifest with the new ID
            let mut updated_manifest = manifest.clone();
            updated_manifest.instance_id = versioned_id.clone();

            // Store the manifest with the versioned ID
            let manifest_type = manifest_type_to_string(&determine_manifest_type(manifest));

            let stored_manifest = StoredManifest {
                _id: None,
                manifest_id: versioned_id.clone(),
                manifest_type,
                manifest: serde_json::to_value(&updated_manifest)
                    .map_err(|e| Error::Serialization(e.to_string()))?,
                created_at: time::OffsetDateTime::now_utc().to_string(),
            };

            self.client
                .post(self.manifest_url(Some(&versioned_id)))
                .json(&stored_manifest)
                .send()
                .map_err(|e| Error::Storage(format!("Failed to store manifest: {e}")))?;

            Ok(versioned_id)
        } else {
            // No existing manifest - store normally
            let manifest_type = manifest_type_to_string(&determine_manifest_type(manifest));

            let stored_manifest = StoredManifest {
                _id: None,
                manifest_id: manifest.instance_id.clone(),
                manifest_type,
                manifest: serde_json::to_value(manifest)
                    .map_err(|e| Error::Serialization(e.to_string()))?,
                created_at: time::OffsetDateTime::now_utc().to_string(),
            };

            self.client
                .post(self.manifest_url(Some(&manifest.instance_id)))
                .json(&stored_manifest)
                .send()
                .map_err(|e| Error::Storage(format!("Failed to store manifest: {e}")))?;

            Ok(manifest.instance_id.clone())
        }
    }

    fn retrieve_manifest(&self, id: &str) -> Result<Manifest> {
        // Parse the ID to find the base UUID part
        let parts: Vec<&str> = id.split(':').collect();
        let uuid_part = if parts.len() >= 3 && parts[0] == "urn" && parts[1] == "c2pa" {
            parts[2] // Extract UUID from urn:c2pa:UUID format
        } else {
            id // Use as-is if not in expected format
        };

        // First try direct retrieval with the given ID
        let response = self
            .client
            .get(format!("{}/manifests/{}", self.base_url, id))
            .send()
            .map_err(|e| Error::Storage(format!("Failed to retrieve manifest: {e}")))?;

        if response.status().is_success() {
            // Found the manifest, parse it
            let stored_manifest: StoredManifest = response
                .json()
                .map_err(|e| Error::Storage(format!("Failed to parse manifest: {e}")))?;

            // Extract the inner manifest
            let manifest_value = stored_manifest
                .manifest
                .get("manifest")
                .ok_or_else(|| Error::Storage("Invalid manifest structure".to_string()))?;

            return serde_json::from_value(manifest_value.clone())
                .map_err(|e| Error::Storage(format!("Failed to parse manifest data: {e}")));
        }

        // If direct lookup failed, try to find all versions
        let list_response = self
            .client
            .get(format!("{}/manifests", self.base_url))
            .send()
            .map_err(|e| Error::Storage(format!("Failed to list manifests: {e}")))?;

        if !list_response.status().is_success() {
            return Err(Error::Storage(format!(
                "Failed to list manifests. Status: {}",
                list_response.status()
            )));
        }

        // Parse the manifest list
        let manifests: Vec<StoredManifest> = list_response
            .json()
            .map_err(|e| Error::Storage(format!("Failed to parse manifests list: {e}")))?;

        // Find all versions of this manifest
        let mut versions: Vec<StoredManifest> = manifests
            .into_iter()
            .filter(|m| m.manifest_id.contains(&format!("urn:c2pa:{uuid_part}:")))
            .collect();

        if versions.is_empty() {
            return Err(Error::Storage(format!("Manifest not found for ID: {id}")));
        }

        // Sort by created_at timestamp (newest first)
        versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Get the latest version
        let latest = &versions[0];

        // Extract the inner manifest
        let manifest_value = latest
            .manifest
            .get("manifest")
            .ok_or_else(|| Error::Storage("Invalid manifest structure".to_string()))?;

        serde_json::from_value(manifest_value.clone())
            .map_err(|e| Error::Storage(format!("Failed to parse manifest data: {e}")))
    }

    fn list_manifests(&self) -> Result<Vec<ManifestMetadata>> {
        let response = self
            .client
            .get(self.manifest_url(None))
            .send()
            .map_err(|e| Error::Storage(format!("Failed to list manifests: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::Storage(format!(
                "Failed to list manifests. Status: {}",
                response.status()
            )));
        }

        let stored_manifests: Vec<StoredManifest> = response
            .json()
            .map_err(|e| Error::Storage(format!("Failed to parse manifests list: {e}")))?;

        Ok(stored_manifests
            .into_iter()
            .map(|m| {
                let title = m
                    .manifest
                    .get("manifest")
                    .and_then(|inner| inner.get("manifest"))
                    .and_then(|manifest| manifest.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                ManifestMetadata {
                    id: m.manifest_id,
                    name: title,
                    manifest_type: match m.manifest_type.as_str() {
                        "dataset" => ManifestType::Dataset,
                        _ => ManifestType::Model,
                    },
                    created_at: m.created_at,
                }
            })
            .collect())
    }

    fn delete_manifest(&self, id: &str) -> Result<()> {
        let response = self
            .client
            .delete(self.manifest_url(Some(id)))
            .send()
            .map_err(|e| Error::Storage(format!("Failed to delete manifest: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::Storage(format!(
                "Failed to delete manifest. Status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    // Added for test suite
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
