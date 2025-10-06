use crate::error::{Error, Result};
use crate::manifest::utils::determine_manifest_type;
use crate::storage::traits::{ManifestMetadata, ManifestType, StorageBackend};
use crate::utils::{safe_create_file, safe_open_file};
use atlas_c2pa_lib::manifest::Manifest;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FilesystemStorage {
    base_path: PathBuf,
}

impl FilesystemStorage {
    pub fn new<P: AsRef<Path>>(url: P) -> Result<Self> {
        // Parse the URL to extract the path
        let path_str = url.as_ref().to_string_lossy();
        let path = if path_str.starts_with("file://") {
            PathBuf::from(path_str.trim_start_matches("file://"))
        } else {
            // Assume it's a direct path if not using file:// scheme
            PathBuf::from(path_str.to_string())
        };

        // Create directory if it doesn't exist
        if !path.exists() {
            create_dir_all(&path)?;
        }

        Ok(Self { base_path: path })
    }

    // Helper to get path for a manifest
    fn manifest_path(&self, id: &str) -> PathBuf {
        // Create a hash of the ID to use as filename
        let digest = Sha256::digest(id.as_bytes());
        let filename = hex::encode(digest);

        self.base_path.join(format!("{filename}.json"))
    }

    // Helper to list all manifest files
    fn list_manifest_files(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.base_path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        Ok(entries)
    }

    // Helper to update index file for quick ID lookups
    fn update_index(&self, id: &str, filename: &str) -> Result<()> {
        let index_path = self.base_path.join("manifest_index.json");

        // Read existing index or create new one
        let mut index: HashMap<String, String> = if index_path.exists() {
            let mut file = safe_open_file(&index_path, false)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Update index
        index.insert(id.to_string(), filename.to_string());

        // Write back to file
        let json = serde_json::to_string_pretty(&index)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let mut file = safe_create_file(&index_path, false)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

impl StorageBackend for FilesystemStorage {
    fn get_base_uri(&self) -> String {
        "file:///".to_string()
    }

    fn store_manifest(&self, manifest: &Manifest) -> Result<String> {
        let manifest_id = manifest.instance_id.clone();
        let path = self.manifest_path(&manifest_id);

        // Serialize to JSON
        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Write to file
        let mut file = safe_create_file(&path, false)?;
        file.write_all(json.as_bytes())?;

        // Update index for quick lookups
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            self.update_index(&manifest_id, filename)?;
        }

        Ok(manifest_id)
    }

    fn retrieve_manifest(&self, id: &str) -> Result<Manifest> {
        let path = self.manifest_path(id);

        if !path.exists() {
            return Err(Error::Storage(format!("Manifest not found: {id}")));
        }

        // Read file
        let mut file = safe_open_file(&path, false)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Deserialize
        serde_json::from_str(&content)
            .map_err(|e| Error::Serialization(format!("Failed to parse manifest: {e}")))
    }

    fn list_manifests(&self) -> Result<Vec<ManifestMetadata>> {
        let mut manifests = Vec::new();

        for path in self.list_manifest_files()? {
            let mut file = safe_open_file(&path, false)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            match serde_json::from_str::<Manifest>(&content) {
                Ok(manifest) => {
                    // Determine manifest type
                    let manifest_type = determine_manifest_type(&manifest);

                    manifests.push(ManifestMetadata {
                        id: manifest.instance_id.clone(),
                        name: manifest.title.clone(),
                        manifest_type,
                        created_at: manifest.created_at.0.to_string(),
                    });
                }
                Err(e) => {
                    // Log but don't fail on unparseable manifest
                    eprintln!("Error parsing manifest at {path:?}: {e}");
                }
            }
        }

        Ok(manifests)
    }

    fn delete_manifest(&self, id: &str) -> Result<()> {
        let path = self.manifest_path(id);

        if !path.exists() {
            return Err(Error::Storage(format!("Manifest not found: {id}")));
        }

        fs::remove_file(&path)?;

        // Update index
        let index_path = self.base_path.join("manifest_index.json");
        if index_path.exists() {
            let mut file = safe_open_file(&index_path, false)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let mut index: HashMap<String, String> =
                serde_json::from_str(&content).unwrap_or_default();

            // Remove entry
            index.remove(id);

            // Write back to file
            let json = serde_json::to_string_pretty(&index)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            let mut file = safe_create_file(&path, false)?;
            file.write_all(json.as_bytes())?;
        }

        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// No conflict impl.
impl FilesystemStorage {
    // List manifests by type
    pub fn list_manifests_by_type(
        &self,
        manifest_type: ManifestType,
    ) -> Result<Vec<ManifestMetadata>> {
        self.list_manifests().map(|all_manifests| {
            all_manifests
                .into_iter()
                .filter(|m| m.manifest_type == manifest_type)
                .collect()
        })
    }

    // Export all manifests to a directory
    pub fn export_all(&self, export_path: PathBuf) -> Result<usize> {
        if !export_path.exists() {
            create_dir_all(&export_path)?;
        }

        let manifests = self.list_manifests()?;
        let mut exported_count = 0;

        for metadata in manifests {
            let manifest = self.retrieve_manifest(&metadata.id)?;
            let json = serde_json::to_string_pretty(&manifest)
                .map_err(|e| Error::Serialization(e.to_string()))?;

            let filename = format!("{}.json", metadata.id.replace(":", "_"));
            let export_file_path = export_path.join(filename);

            let mut file = safe_create_file(&export_file_path, false)?;
            file.write_all(json.as_bytes())?;

            exported_count += 1;
        }

        Ok(exported_count)
    }

    // Import manifests from a directory
    pub fn import_from_directory(&self, import_path: PathBuf) -> Result<usize> {
        if !import_path.exists() || !import_path.is_dir() {
            return Err(Error::Storage(format!(
                "Import path does not exist or is not a directory: {import_path:?}"
            )));
        }

        let entries = fs::read_dir(import_path)?.filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                Some(path)
            } else {
                None
            }
        });

        let mut imported_count = 0;

        for path in entries {
            let mut file = safe_open_file(&path, false)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            match serde_json::from_str::<Manifest>(&content) {
                Ok(manifest) => {
                    self.store_manifest(&manifest)?;
                    imported_count += 1;
                }
                Err(e) => {
                    eprintln!("Error importing manifest from {path:?}: {e}");
                }
            }
        }

        Ok(imported_count)
    }

    // Get manifest file size
    pub fn get_manifest_size(&self, id: &str) -> Result<u64> {
        let path = self.manifest_path(id);

        if !path.exists() {
            return Err(Error::Storage(format!("Manifest not found: {id}")));
        }

        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }

    // Get total storage size
    pub fn get_total_storage_size(&self) -> Result<u64> {
        let mut total_size = 0;

        for path in self.list_manifest_files()? {
            let metadata = fs::metadata(path)?;
            total_size += metadata.len();
        }

        // Include index file if it exists
        let index_path = self.base_path.join("manifest_index.json");
        if index_path.exists() {
            let metadata = fs::metadata(index_path)?;
            total_size += metadata.len();
        }

        Ok(total_size)
    }

    // Check if a manifest exists
    pub fn manifest_exists(&self, id: &str) -> bool {
        self.manifest_path(id).exists()
    }

    // Backup all manifests to a directory
    pub fn backup(&self, backup_path: PathBuf) -> Result<()> {
        // Simplified version just copies files
        if !backup_path.exists() {
            create_dir_all(&backup_path)?;
        }

        for path in self.list_manifest_files()? {
            if let Some(filename) = path.file_name() {
                let dest_path = backup_path.join(filename);
                fs::copy(path, dest_path)?;
            }
        }

        // Copy index file if it exists
        let index_path = self.base_path.join("manifest_index.json");
        if index_path.exists() {
            let dest_path = backup_path.join("manifest_index.json");
            fs::copy(index_path, dest_path)?;
        }

        Ok(())
    }
}
