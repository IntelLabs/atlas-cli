use crate::error::Error;
use crate::error::Result;
use atlas_c2pa_lib::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt;
use std::path::PathBuf;

/// Represents metadata about a stored manifest
///
/// # Examples
///
/// ```
/// use atlas_cli::storage::traits::{ManifestMetadata, ManifestType};
///
/// let metadata = ManifestMetadata {
///     id: "model-123".to_string(),
///     name: "My Model".to_string(),
///     manifest_type: ManifestType::Model,
///     created_at: "2025-01-23T12:00:00Z".to_string(),
/// };
///
/// assert_eq!(metadata.id, "model-123");
/// assert_eq!(metadata.manifest_type, ManifestType::Model);
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub id: String,
    pub name: String,
    pub manifest_type: ManifestType,
    pub created_at: String,
}

pub trait StorageBackend {
    fn get_base_uri(&self) -> String;
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
    Unknown,
}

/// Represents the location and verification info for an artifact
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::storage::traits::ArtifactLocation;
/// use std::path::PathBuf;
///
/// // Create from a file path
/// let path = PathBuf::from("model.onnx");
/// let location = ArtifactLocation::new(path).unwrap();
///
/// // Verify the file hasn't changed
/// assert!(location.verify().unwrap());
/// ```
///
/// ```
/// use atlas_cli::storage::traits::ArtifactLocation;
/// use std::path::PathBuf;
///
/// // Create manually
/// let location = ArtifactLocation {
///     url: "file:///path/to/file".to_string(),
///     file_path: Some(PathBuf::from("/path/to/file")),
///     hash: "a".repeat(64),
/// };
///
/// assert!(location.file_path.is_some());
/// ```
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
            ManifestType::Unknown => write!(f, "Unknown"),
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
