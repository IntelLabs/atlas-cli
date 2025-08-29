use crate::storage::traits::StorageBackend;
use atlas_c2pa_lib::cose::HashAlgorithm;
use std::path::PathBuf;

pub struct ManifestCreationConfig {
    pub paths: Vec<PathBuf>,
    pub ingredient_names: Vec<String>,
    pub name: String,
    pub author_org: Option<String>,
    pub author_name: Option<String>,
    pub description: Option<String>,
    pub linked_manifests: Option<Vec<String>>,
    pub storage: Option<&'static dyn StorageBackend>,
    pub print: bool,
    pub output_encoding: String,
    pub key_path: Option<PathBuf>,
    pub hash_alg: HashAlgorithm,
    pub with_cc: bool,
    // Software-specific fields
    pub software_type: Option<String>,
    pub version: Option<String>,
    // Generic custom_fields for extensions
    pub custom_fields: Option<serde_json::Value>,
}

impl ManifestCreationConfig {
    pub fn clone_without_storage(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            ingredient_names: self.ingredient_names.clone(),
            name: self.name.clone(),
            author_org: self.author_org.clone(),
            author_name: self.author_name.clone(),
            description: self.description.clone(),
            linked_manifests: self.linked_manifests.clone(),
            storage: self.storage,
            print: self.print,
            output_encoding: self.output_encoding.clone(),
            key_path: self.key_path.clone(),
            hash_alg: self.hash_alg.clone(),
            with_cc: self.with_cc,
            software_type: self.software_type.clone(),
            version: self.version.clone(),
            custom_fields: self.custom_fields.clone(),
        }
    }
}
