use crate::error::Error;
use crate::error::Result;
use c2pa_ml::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub id: String,
    pub name: String,
    pub manifest_type: ManifestType,
    pub created_at: String,
}

pub trait StorageBackend {
    fn store_manifest(&self, manifest: &Manifest) -> Result<String>;
    fn retrieve_manifest(&self, id: &str) -> Result<Manifest>;
    fn list_manifests(&self) -> Result<Vec<ManifestMetadata>>;
    fn delete_manifest(&self, id: &str) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ManifestType {
    Dataset,
    Model,
    Software,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArtifactLocation {
    pub url: String,
    pub file_path: Option<PathBuf>,
    pub hash: String,
}

impl fmt::Display for ManifestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestType::Dataset => write!(f, "Dataset"),
            ManifestType::Model => write!(f, "Model"),
            ManifestType::Software => write!(f, "Software"),
        }
    }
}

impl ArtifactLocation {
    pub fn new(path: PathBuf) -> Result<Self> {
        let hash = crate::hash::calculate_file_hash(&path)?;
        let url = format!("file://{}", path.to_string_lossy());

        Ok(Self {
            url,
            file_path: Some(path),
            hash,
        })
    }

    pub fn verify(&self) -> Result<bool> {
        match &self.file_path {
            Some(path) => {
                let current_hash = crate::hash::calculate_file_hash(path)?;
                Ok(current_hash == self.hash)
            }
            None => Err(Error::Validation(
                "No file path available for verification".to_string(),
            )),
        }
    }
}
