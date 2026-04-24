use super::common::create_default_claim;
use crate::in_toto::dsse::Envelope;
use crate::signing;
use crate::signing::signable::Signable;
use crate::storage::rekor::RekorStorage;
use atlas_c2pa_lib::cose::HashAlgorithm;
use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
use atlas_c2pa_lib::manifest::Manifest;
use time::OffsetDateTime;
use uuid::Uuid;

fn generate_temp_key_and_cert() -> (
    signing::SecurePrivateKey,
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    use openssl::asn1::Asn1Time;
    use openssl::bn::BigNum;
    use openssl::hash::MessageDigest;
    use openssl::x509::{X509Builder, X509NameBuilder};

    let (secure_key, tmp_dir) = signing::test_utils::generate_temp_key().unwrap();
    let key_path = tmp_dir.path().join("test_key.pem");

    let mut name_builder = X509NameBuilder::new().unwrap();
    name_builder
        .append_entry_by_text("CN", "atlas-cli-test")
        .unwrap();
    let name = name_builder.build();

    let mut builder = X509Builder::new().unwrap();
    builder.set_version(2).unwrap();
    let serial = BigNum::from_u32(1)
        .and_then(|bn| bn.to_asn1_integer())
        .unwrap();
    builder.set_serial_number(&serial).unwrap();
    builder.set_subject_name(&name).unwrap();
    builder.set_issuer_name(&name).unwrap();
    builder.set_pubkey(secure_key.as_pkey()).unwrap();
    builder
        .set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    builder
        .set_not_after(&Asn1Time::days_from_now(1).unwrap())
        .unwrap();
    builder
        .sign(secure_key.as_pkey(), MessageDigest::sha256())
        .unwrap();

    let cert = builder.build();
    let cert_path = tmp_dir.path().join("test_cert.pem");
    let cert_pem = cert.to_pem().unwrap();
    std::fs::write(&cert_path, &cert_pem).unwrap();

    (secure_key, tmp_dir, key_path, cert_path)
}

#[test]
#[ignore] // Requires network access to rekor.sigstore.dev
fn test_rekor_dsse_store_and_retrieve() {
    let (_secure_key, _tmp_dir, key_path, cert_path) = generate_temp_key_and_cert();

    let payload = br#"{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"test-artifact","digest":{"sha256":"abc123"}}],"predicateType":"https://example.com/test","predicate":{}}"#;
    let mut envelope = Envelope::new(
        &payload.to_vec(),
        "application/vnd.in-toto+json".to_string(),
    );
    envelope
        .sign(key_path.clone(), HashAlgorithm::Sha256)
        .unwrap();
    assert!(envelope.validate());

    let storage = RekorStorage::new().unwrap();
    let log_entry = storage.store_dsse_envelope(&envelope, &cert_path).unwrap();

    assert!(!log_entry.uuid.to_string().is_empty());
    assert!(log_entry.log_index >= 0);
    assert!(log_entry.integrated_time > 0);
    println!(
        "Rekor entry created: uuid={}, log_index={}",
        log_entry.uuid, log_entry.log_index
    );

    let retrieved = storage
        .get_rekor_entry(&log_entry.uuid.to_string())
        .unwrap();
    assert_eq!(retrieved.uuid.to_string(), log_entry.uuid.to_string());
    assert_eq!(retrieved.log_index, log_entry.log_index);
    println!("Rekor entry retrieved successfully");
}

#[test]
#[ignore] // Requires network access to rekor.sigstore.dev
fn test_rekor_storage_backend_store_manifest() {
    use crate::storage::traits::StorageBackend;

    let (_secure_key, _tmp_dir, key_path, cert_path) = generate_temp_key_and_cert();

    let storage = RekorStorage::new()
        .unwrap()
        .with_key(Some(key_path))
        .with_cert(Some(cert_path));

    let manifest = Manifest {
        claim_generator: "atlas-cli-test".to_string(),
        title: "Rekor Integration Test".to_string(),
        instance_id: Uuid::new_v4().to_string(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    let uuid = storage.store_manifest(&manifest).unwrap();
    assert!(!uuid.is_empty());
    println!("Manifest stored in Rekor with UUID: {uuid}");

    let entry = storage.get_rekor_entry(&uuid).unwrap();
    assert_eq!(entry.uuid.to_string(), uuid);
    println!("Verified entry exists at log_index={}", entry.log_index);

    let manifest_json = serde_json::to_vec(&manifest).unwrap();
    let result = storage.verify_manifest(&manifest_json, &uuid).unwrap();
    assert!(result.payload_hash_match, "Payload hash should match");
    assert!(result.signature_valid, "Signature should be valid");
    println!(
        "Verification passed: hash_match={}, sig_valid={}",
        result.payload_hash_match, result.signature_valid
    );

    let mut tampered = manifest_json.clone();
    tampered[0] = b'X';
    let bad_result = storage.verify_manifest(&tampered, &uuid).unwrap();
    assert!(
        !bad_result.payload_hash_match,
        "Tampered payload hash should not match"
    );
    assert!(
        !bad_result.signature_valid,
        "Tampered signature should not be valid"
    );
    println!("Tampered manifest correctly rejected");
}

#[test]
#[ignore] // Requires network access to rekor.sigstore.dev
fn test_rekor_store_and_verify_from_disk() {
    use crate::storage::traits::StorageBackend;

    let (_secure_key, _tmp_dir, key_path, cert_path) = generate_temp_key_and_cert();

    let storage = RekorStorage::new()
        .unwrap()
        .with_key(Some(key_path))
        .with_cert(Some(cert_path));

    let manifest = Manifest {
        claim_generator: "atlas-cli-test".to_string(),
        title: "Rekor Disk Verification Test".to_string(),
        instance_id: Uuid::new_v4().to_string(),
        ingredients: Vec::new(),
        claim: create_default_claim(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
        is_active: true,
    };

    let manifest_json = serde_json::to_vec(&manifest).unwrap();
    let manifest_path = std::env::current_dir()
        .unwrap()
        .join("test_manifest_rekor.json");
    std::fs::write(&manifest_path, &manifest_json).unwrap();
    println!("Manifest saved to: {}", manifest_path.display());

    let uuid = storage.store_manifest(&manifest).unwrap();
    println!("Manifest stored in Rekor with UUID: {uuid}");

    println!("\n=== To verify, run: ===");
    println!(
        "cargo run -- rekor verify --uuid {uuid} --manifest {}",
        manifest_path.display()
    );
    println!("=======================\n");

    let result = storage.verify_manifest(&manifest_json, &uuid).unwrap();
    assert!(result.payload_hash_match);
    assert!(result.signature_valid);
    println!("In-test verification passed");
}
