use crate::error::Result;
use crate::manifest::common::{AssetKind, create_manifest};
use crate::manifest::config::ManifestCreationConfig;
use crate::storage::traits::StorageBackend;
use crate::tests::common::MockStorageBackend;
use crate::utils::safe_create_file;
use atlas_c2pa_lib::cose::HashAlgorithm;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_manifest_with_cc_attestation() -> Result<()> {
    // Create a temporary directory and test file
    let dir = tempdir()?;
    let test_file_path = dir.path().join("test_model.onnx");

    // Create a test file
    {
        let mut file = safe_create_file(&test_file_path, false)?;
        file.write_all(b"test model data")?;
    }

    // Setup test parameters
    let paths = vec![test_file_path];
    let ingredient_names = vec!["Test Model".to_string()];
    let name = "Test Model with Attestation".to_string();
    let with_cc = true; // Enable CC attestation

    // Create empty storage and leak it to make it 'static
    let storage_box = Box::new(MockStorageBackend::new_empty());
    let storage = Box::leak(storage_box);

    // Create the manifest configuration
    let config = ManifestCreationConfig {
        paths,
        ingredient_names,
        name,
        author_org: Some("Test Organization".to_string()),
        author_name: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        linked_manifests: None,
        storage: Some(storage),
        print: true,
        output_encoding: "json".to_string(),
        key_path: None,
        hash_alg: HashAlgorithm::Sha384,
        with_cc,
        software_type: None,
        version: None,
        custom_fields: None,
    };

    // Create the manifest with CC attestation enabled
    let result = create_manifest(config, AssetKind::Model);

    // Verify that the manifest was created successfully
    assert!(
        result.is_ok(),
        "Failed to create manifest with CC attestation: {:?}",
        result.err()
    );

    // Clean up
    dir.close()?;

    Ok(())
}

#[test]
fn test_manifest_without_cc_attestation() -> Result<()> {
    // Create a temporary directory and test file
    let dir = tempdir()?;
    let test_file_path = dir.path().join("test_model.onnx");

    // Create a test file
    {
        let mut file = safe_create_file(&test_file_path, false)?;
        file.write_all(b"test model data")?;
    }

    // Setup test parameters
    let paths = vec![test_file_path];
    let ingredient_names = vec!["Test Model".to_string()];
    let name = "Test Model without Attestation".to_string();
    let with_cc = false; // Disable CC attestation

    // Create empty storage and leak it to make it 'static
    let storage_box = Box::new(MockStorageBackend::new_empty());
    let storage = Box::leak(storage_box);

    // Create the manifest configuration
    let config = ManifestCreationConfig {
        paths,
        ingredient_names,
        name,
        author_org: Some("Test Organization".to_string()),
        author_name: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        linked_manifests: None,
        storage: Some(storage),
        print: true,
        output_encoding: "json".to_string(),
        key_path: None,
        hash_alg: HashAlgorithm::Sha384,
        with_cc,
        software_type: None,
        version: None,
        custom_fields: None,
    };

    // Create the manifest without CC attestation
    let result = create_manifest(config, AssetKind::Model);

    // Verify that the manifest was created successfully
    assert!(
        result.is_ok(),
        "Failed to create manifest without CC attestation: {:?}",
        result.err()
    );

    // Clean up
    dir.close()?;

    Ok(())
}

// Test comparing manifests with and without attestation
#[test]
fn test_compare_manifests_with_and_without_attestation() -> Result<()> {
    // Create a temporary directory and test file
    let dir = tempdir()?;
    let test_file_path = dir.path().join("test_model.onnx");

    // Create a test file
    {
        let mut file = safe_create_file(&test_file_path, false)?;
        file.write_all(b"test model data")?;
    }

    // Setup common parameters
    let paths = vec![test_file_path.clone()];
    let ingredient_names = vec!["Test Model".to_string()];
    let name = "Attestation Test Model".to_string();

    // Create storages and leak them to make them 'static
    let storage_with_cc_box = Box::new(MockStorageBackend::new_empty());
    let storage_with_cc = Box::leak(storage_with_cc_box);

    let storage_without_cc_box = Box::new(MockStorageBackend::new_empty());
    let storage_without_cc = Box::leak(storage_without_cc_box);

    // Create manifests with different attestation settings
    // With CC attestation
    let config_with_cc = ManifestCreationConfig {
        paths: paths.clone(),
        ingredient_names: ingredient_names.clone(),
        name: name.clone(),
        author_org: Some("Test Organization".to_string()),
        author_name: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        linked_manifests: None,
        storage: Some(storage_with_cc),
        print: false,
        output_encoding: "json".to_string(),
        key_path: None,
        hash_alg: HashAlgorithm::Sha384,
        with_cc: true,
        software_type: None,
        version: None,
        custom_fields: None,
    };
    create_manifest(config_with_cc, AssetKind::Model)?;

    // Without CC attestation
    let config_without_cc = ManifestCreationConfig {
        paths,
        ingredient_names,
        name,
        author_org: Some("Test Organization".to_string()),
        author_name: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        linked_manifests: None,
        storage: Some(storage_without_cc),
        print: false,
        output_encoding: "json".to_string(),
        key_path: None,
        hash_alg: HashAlgorithm::Sha384,
        with_cc: false,
        software_type: None,
        version: None,
        custom_fields: None,
    };
    create_manifest(config_without_cc, AssetKind::Model)?;

    // List manifests from both storages
    let manifests_with_cc = storage_with_cc.list_manifests()?;
    let manifests_without_cc = storage_without_cc.list_manifests()?;

    // Verify we got manifests in both cases
    assert!(
        !manifests_with_cc.is_empty(),
        "No manifests created with CC attestation"
    );
    assert!(
        !manifests_without_cc.is_empty(),
        "No manifests created without CC attestation"
    );

    // Clean up
    dir.close()?;

    Ok(())
}
