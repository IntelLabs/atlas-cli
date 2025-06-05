use super::common::{MockStorageBackend, create_default_claim};
use crate::cli::commands::DatasetCommands;
use crate::cli::handlers::handle_dataset_command;
use crate::error::Result;
use crate::storage::filesystem::FilesystemStorage;
use crate::storage::traits::ArtifactLocation;
use crate::storage::traits::StorageBackend;
use crate::utils::safe_create_file;
use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
use atlas_c2pa_lib::manifest::Manifest;
use sha2::Digest;
use std::fs;
use std::io::Write;
use tempfile::tempdir;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn test_artifact_location() -> Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("test.txt");

    // Create a test file
    let mut file = safe_create_file(&file_path, false)?;
    file.write_all(b"test content")?;

    // Create and verify ArtifactLocation
    let location = ArtifactLocation::new(file_path.clone())?;

    assert!(location.url.starts_with("file://"));
    assert_eq!(location.file_path.as_ref().unwrap(), &file_path);
    assert!(location.verify()?);

    // Modify file and verify hash mismatch
    let mut file = safe_create_file(&file_path, false)?;
    file.write_all(b"modified content")?;

    assert!(!location.verify()?);

    Ok(())
}

#[test]
fn test_mock_storage() -> Result<()> {
    // Create test manifest
    let manifest_id = format!("test_manifest_{}", Uuid::new_v4());
    let manifest = Manifest {
        claim_generator: "test".to_string(),
        title: "Test Manifest".to_string(),
        instance_id: manifest_id.clone(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    // Initialize storage
    let storage = MockStorageBackend::new(manifest.clone());

    // Test retrieval
    let retrieved = storage.retrieve_manifest(&manifest_id)?;
    assert_eq!(retrieved.instance_id, manifest_id);

    // Test list
    let manifests = storage.list_manifests()?;
    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].id, manifest_id);

    // Test delete
    assert!(storage.delete_manifest(&manifest_id).is_ok());
    assert!(storage.retrieve_manifest(&manifest_id).is_err());

    Ok(())
}

#[test]
fn test_filesystem_storage() -> Result<()> {
    // Create a temporary directory for storage
    let dir = tempdir()?;
    println!("Storage path: {:?}", dir.path());

    // Initialize filesystem storage
    let fs_storage = FilesystemStorage::new(dir.path().to_string_lossy().to_string())?;

    // Create a test manifest
    let manifest_id = format!("test_manifest_{}", Uuid::new_v4());
    println!("Created manifest ID: {manifest_id}");

    let manifest = Manifest {
        claim_generator: "test".to_string(),
        title: "Test Filesystem Storage".to_string(),
        instance_id: manifest_id.clone(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    // Store the manifest
    let stored_id = fs_storage.store_manifest(&manifest)?;
    println!("Stored manifest with ID: {stored_id}");
    assert_eq!(stored_id, manifest_id);

    // Retrieve the manifest
    let retrieved = fs_storage.retrieve_manifest(&manifest_id)?;
    println!("Retrieved manifest with title: {}", retrieved.title);
    assert_eq!(retrieved.instance_id, manifest_id);
    assert_eq!(retrieved.title, "Test Filesystem Storage");

    // List manifests
    let manifests = fs_storage.list_manifests()?;
    println!("Found {} manifests in storage", manifests.len());
    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].id, manifest_id);

    // Delete the manifest
    fs_storage.delete_manifest(&manifest_id)?;
    println!("Deleted manifest with ID: {manifest_id}");

    // Verify it's no longer retrievable
    assert!(fs_storage.retrieve_manifest(&manifest_id).is_err());
    println!("Verified manifest is no longer accessible");

    Ok(())
}

#[test]
fn test_filesystem_storage_extended() -> Result<()> {
    // Create a temporary directory for storage
    let dir = tempdir()?;
    println!("Storage path: {:?}", dir.path());

    // Initialize filesystem storage
    let fs_storage = FilesystemStorage::new(dir.path().to_string_lossy().to_string())?;

    // Create a test manifest
    let manifest_id = format!("test_manifest_{}", Uuid::new_v4());
    println!("Created manifest ID: {manifest_id}");

    let manifest = Manifest {
        claim_generator: "test".to_string(),
        title: "Test Filesystem Storage".to_string(),
        instance_id: manifest_id.clone(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    // Store the manifest
    fs_storage.store_manifest(&manifest)?;

    // Test manifest_exists functionality
    assert!(
        fs_storage.manifest_exists(&manifest_id),
        "Should report manifest exists"
    );
    assert!(
        !fs_storage.manifest_exists("nonexistent_id"),
        "Should report nonexistent manifest doesn't exist"
    );
    println!("Verified manifest_exists functionality");

    // Test manifest size functionality
    let size = fs_storage.get_manifest_size(&manifest_id)?;
    assert!(size > 0, "Manifest file size should be greater than 0");
    println!("Manifest size: {size} bytes");

    // Test backup functionality
    let backup_dir = tempdir()?;
    println!("Backup directory: {:?}", backup_dir.path());
    fs_storage.backup(backup_dir.path().to_path_buf())?;

    // Verify backup contains files
    let backup_count = fs::read_dir(backup_dir.path())?
        .filter_map(|entry| entry.ok())
        .count();
    assert!(backup_count > 0, "Backup directory should contain files");
    println!("Backup contains {backup_count} files");

    // Test export functionality
    let export_dir = tempdir()?;
    println!("Export directory: {:?}", export_dir.path());
    let export_count = fs_storage.export_all(export_dir.path().to_path_buf())?;
    assert_eq!(export_count, 1, "Should export 1 manifest");
    println!("Exported {export_count} manifests");

    // Test total storage size
    let total_size = fs_storage.get_total_storage_size()?;
    assert!(
        total_size > 0,
        "Total storage size should be greater than 0"
    );
    println!("Total storage size: {total_size} bytes");

    Ok(())
}

#[test]
fn test_filesystem_storage_linking() -> Result<()> {
    // Create a temporary directory for storage
    let dir = tempdir()?;
    println!("Storage path: {:?}", dir.path());

    // Initialize filesystem storage
    let fs_storage = FilesystemStorage::new(dir.path().to_string_lossy().to_string())?;

    // Create two test manifests
    let dataset_id = format!("dataset_{}", Uuid::new_v4());
    let model_id = format!("model_{}", Uuid::new_v4());

    println!("Created dataset ID: {dataset_id}");
    println!("Created model ID: {model_id}");

    // Create dataset manifest
    let dataset_manifest = Manifest {
        claim_generator: "test".to_string(),
        title: "Test Dataset".to_string(),
        instance_id: dataset_id.clone(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    // Create model manifest
    let mut model_manifest = Manifest {
        claim_generator: "test".to_string(),
        title: "Test Model".to_string(),
        instance_id: model_id.clone(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    // Store both manifests
    fs_storage.store_manifest(&dataset_manifest)?;
    fs_storage.store_manifest(&model_manifest)?;
    println!("Stored both manifests");

    // Create a cross-reference from model to dataset
    let dataset_json = serde_json::to_string(&dataset_manifest)?;
    let dataset_hash = hex::encode(sha2::Sha256::digest(dataset_json.as_bytes()));

    let cross_ref = atlas_c2pa_lib::cross_reference::CrossReference {
        manifest_url: dataset_id.clone(),
        manifest_hash: dataset_hash,
        media_type: Some("application/json".to_string()),
    };

    // Add cross-reference to model manifest
    model_manifest.cross_references.push(cross_ref);
    println!("Added cross-reference from model to dataset");

    // Update model manifest in storage
    fs_storage.store_manifest(&model_manifest)?;
    println!("Updated model manifest with cross-reference");

    // Retrieve the model manifest and verify cross-reference
    let retrieved_model = fs_storage.retrieve_manifest(&model_id)?;

    assert!(
        !retrieved_model.cross_references.is_empty(),
        "Model should have cross-references"
    );
    assert_eq!(
        retrieved_model.cross_references[0].manifest_url, dataset_id,
        "Cross-reference should point to dataset"
    );

    println!("Verified cross-reference exists in retrieved model manifest");

    // Verify cross-reference hash by retrieving dataset
    let retrieved_dataset = fs_storage.retrieve_manifest(&dataset_id)?;
    let retrieved_dataset_json = serde_json::to_string(&retrieved_dataset)?;
    let calculated_hash = hex::encode(sha2::Sha256::digest(retrieved_dataset_json.as_bytes()));

    assert_eq!(
        retrieved_model.cross_references[0].manifest_hash, calculated_hash,
        "Cross-reference hash should match calculated hash"
    );

    println!("Verified cross-reference hash is correct");

    // Test deleting dataset and checking that model still has cross-reference
    // (though it would now shn be invalid)
    fs_storage.delete_manifest(&dataset_id)?;
    println!("Deleted dataset manifest");

    let final_model = fs_storage.retrieve_manifest(&model_id)?;
    assert!(
        !final_model.cross_references.is_empty(),
        "Model should still have cross-reference even after dataset deletion"
    );

    println!("Verified model still has cross-reference after dataset deletion");

    Ok(())
}

#[test]
fn test_cli_handler_storage_selection() -> Result<()> {
    // Create a mock DatasetCommands::List command with different storage types
    let test_cases = vec![
        ("database", "http://localhost:8080"),
        ("local-fs", "/tmp/storage"),
    ];

    for (storage_type, storage_url) in test_cases {
        // Mock the CLI command
        let cmd = DatasetCommands::List {
            storage_type: Box::new(storage_type.to_string()),
            storage_url: Box::new(storage_url.to_string()),
        };

        let result = handle_dataset_command(cmd);

        // The connection errors are expected, we're just making sure we don't get type errors
        match result {
            Ok(_) => println!("✓ Command succeeded with storage_type: {storage_type}"),
            Err(e) => {
                // Check if it's an expected error
                let err_string = e.to_string();

                // These are all expected connection errors
                let expected_errors = [
                    "Connection refused",
                    "No such file or directory",
                    "Failed to connect",
                    "Failed to list manifests",
                    "tcp connect error",
                    "Failed to retrieve",
                    "error sending request",
                ];

                if expected_errors.iter().any(|msg| err_string.contains(msg)) {
                    println!(
                        "✓ Command failed with expected connection error for storage_type '{storage_type}': {err_string}"
                    );
                } else {
                    // Unexpected error - likely due to incorrect storage backend selection
                    panic!(
                        "Command failed with unexpected error for storage_type '{storage_type}': {err_string}"
                    );
                }
            }
        }
    }

    Ok(())
}
