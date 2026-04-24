use crate::error::{Error, Result};
use crate::in_toto::dsse::Envelope;
use crate::signing::signable::Signable;
use crate::storage::traits::{ManifestMetadata, StorageBackend};
use atlas_c2pa_lib::cose::HashAlgorithm;
use atlas_c2pa_lib::manifest::Manifest;
use sha2::{Digest, Sha256};
use sigstore_rekor::body::RekorEntryBody;
use sigstore_rekor::{DsseEntry, LogEntry, RekorClient};
use sigstore_types::DerCertificate;
use std::path::{Path, PathBuf};

const MANIFEST_PAYLOAD_TYPE: &str = "application/vnd.atlas-cli.manifest+json";

/// How the certificate is obtained for Rekor submissions.
pub enum CertificateSource {
    /// Load from a PEM file on disk.
    File(PathBuf),
    /// Obtain from Fulcio using an OIDC identity token.
    /// The DSSE envelope will be signed with an ephemeral ECDSA P-256 key.
    Fulcio(String),
}

pub struct RekorStorage {
    client: RekorClient,
    runtime: tokio::runtime::Runtime,
    base_url: String,
    key_path: Option<PathBuf>,
    cert_source: Option<CertificateSource>,
}

impl RekorStorage {
    pub fn new() -> Result<Self> {
        Self::new_with_url("https://rekor.sigstore.dev".to_string())
    }

    pub fn new_with_url(url: String) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Storage(format!("Failed to create async runtime: {e}")))?;
        let client = RekorClient::new(&url);
        Ok(RekorStorage {
            client,
            runtime,
            base_url: url,
            key_path: None,
            cert_source: None,
        })
    }

    pub fn with_key(mut self, key_path: Option<PathBuf>) -> Self {
        self.key_path = key_path;
        self
    }

    pub fn with_cert(mut self, cert_path: Option<PathBuf>) -> Self {
        if let Some(path) = cert_path {
            self.cert_source = Some(CertificateSource::File(path));
        }
        self
    }

    pub fn with_fulcio(mut self, oidc_token: String) -> Self {
        self.cert_source = Some(CertificateSource::Fulcio(oidc_token));
        self
    }

    fn resolve_cert_and_sign_envelope(
        &self,
        manifest_json: &[u8],
    ) -> Result<(Envelope, DerCertificate)> {
        match &self.cert_source {
            Some(CertificateSource::File(cert_path)) => {
                let key_path = self.key_path.as_ref().ok_or_else(|| {
                    Error::Storage(
                        "Signing key required with --cert. Use --key to specify a private key."
                            .to_string(),
                    )
                })?;
                let cert = load_cert_from_file(cert_path)?;
                let mut envelope =
                    Envelope::new(&manifest_json.to_vec(), MANIFEST_PAYLOAD_TYPE.to_string());
                envelope.sign(key_path.clone(), HashAlgorithm::Sha256)?;
                Ok((envelope, cert))
            }
            Some(CertificateSource::Fulcio(oidc_token)) => self
                .runtime
                .block_on(sign_with_fulcio(manifest_json, oidc_token)),
            None => Err(Error::Storage(
                "Certificate required for Rekor storage. \
                 Use --cert <path> or --fulcio --oidc-token <token>."
                    .to_string(),
            )),
        }
    }

    /// Submit a pre-built DSSE envelope to Rekor with a certificate loaded from file.
    pub fn store_dsse_envelope(&self, envelope: &Envelope, cert_path: &Path) -> Result<LogEntry> {
        let cert = load_cert_from_file(cert_path)?;
        let sigstore_dsse = envelope.to_sigstore_dsse();
        let entry = DsseEntry::new(&sigstore_dsse, &cert);
        self.runtime
            .block_on(self.client.create_dsse_entry(entry))
            .map_err(|e| Error::Storage(format!("Rekor submission failed: {e}")))
    }

    /// Retrieve a log entry from Rekor by its UUID.
    pub fn get_rekor_entry(&self, uuid: &str) -> Result<LogEntry> {
        self.runtime
            .block_on(self.client.get_entry_by_uuid(uuid))
            .map_err(|e| Error::Storage(format!("Rekor retrieval failed: {e}")))
    }

    /// Search Rekor entries by SHA-256 hash (hex-encoded).
    pub fn search_by_hash(&self, sha256_hex: &str) -> Result<Vec<String>> {
        self.runtime
            .block_on(self.client.search_by_hash(sha256_hex))
            .map_err(|e| Error::Storage(format!("Rekor search failed: {e}")))
    }

    /// Verify a local manifest against a Rekor transparency log entry.
    ///
    /// Checks:
    /// 1. Payload hash matches the SHA-256 of the provided manifest bytes
    /// 2. DSSE signature is valid against the certificate stored in the entry
    pub fn verify_manifest(&self, manifest_bytes: &[u8], uuid: &str) -> Result<RekorVerifyResult> {
        let log_entry = self.get_rekor_entry(uuid)?;
        let body_b64 = log_entry.body.to_base64();

        let body = RekorEntryBody::from_base64_json(&body_b64, "dsse", "0.0.1")
            .map_err(|e| Error::Storage(format!("Failed to parse Rekor entry body: {e}")))?;

        let (payload_hash_hex, signatures, cert_der) = match body {
            RekorEntryBody::DsseV001(b) => {
                let hash = b.spec.payload_hash.value;
                let sigs = b.spec.signatures;
                let cert = sigs
                    .first()
                    .ok_or_else(|| Error::Storage("No signatures in Rekor entry".to_string()))?
                    .to_certificate()
                    .map_err(|e| {
                        Error::Storage(format!("Failed to parse certificate from entry: {e}"))
                    })?;
                (hash, sigs, cert)
            }
            _ => {
                return Err(Error::Storage(
                    "Unsupported Rekor entry type for verification. Expected DSSE v0.0.1."
                        .to_string(),
                ));
            }
        };

        let local_hash = hex::encode(Sha256::digest(manifest_bytes));
        let payload_hash_match = local_hash == payload_hash_hex;

        let signature_valid = verify_dsse_signature(manifest_bytes, &signatures, &cert_der);

        let cert_info = sigstore_crypto::parse_certificate_info(cert_der.as_bytes()).ok();
        let signer_identity = cert_info.as_ref().and_then(|c| c.identity.clone());

        Ok(RekorVerifyResult {
            payload_hash_match,
            signature_valid,
            log_index: log_entry.log_index,
            integrated_time: log_entry.integrated_time,
            signer_identity,
        })
    }
}

impl StorageBackend for RekorStorage {
    fn get_base_uri(&self) -> String {
        self.base_url.clone()
    }

    fn store_manifest(&self, manifest: &Manifest) -> Result<String> {
        let manifest_json = serde_json::to_vec(manifest)
            .map_err(|e| Error::Serialization(format!("Failed to serialize manifest: {e}")))?;

        let (envelope, cert) = self.resolve_cert_and_sign_envelope(&manifest_json)?;
        let sigstore_dsse = envelope.to_sigstore_dsse();
        let entry = DsseEntry::new(&sigstore_dsse, &cert);
        let log_entry = self
            .runtime
            .block_on(self.client.create_dsse_entry(entry))
            .map_err(|e| Error::Storage(format!("Rekor submission failed: {e}")))?;
        Ok(log_entry.uuid.to_string())
    }

    fn retrieve_manifest(&self, _id: &str) -> Result<Manifest> {
        Err(Error::Storage(
            "Rekor is a transparency log that stores hashes, not full content. \
             Use get_rekor_entry() to retrieve log entry metadata."
                .to_string(),
        ))
    }

    fn list_manifests(&self) -> Result<Vec<ManifestMetadata>> {
        Err(Error::Storage(
            "Rekor does not support listing all entries. \
             Use search_by_hash() to find entries by artifact hash."
                .to_string(),
        ))
    }

    fn delete_manifest(&self, _id: &str) -> Result<()> {
        Err(Error::Storage(
            "Rekor is an immutable transparency log. Entries cannot be deleted.".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct RekorVerifyResult {
    pub payload_hash_match: bool,
    pub signature_valid: bool,
    pub log_index: i64,
    pub integrated_time: i64,
    pub signer_identity: Option<String>,
}

fn verify_dsse_signature(
    payload: &[u8],
    signatures: &[sigstore_rekor::body::DsseV001Signature],
    cert_der: &DerCertificate,
) -> bool {
    use crate::in_toto::dsse::pae;

    let cert_info = match sigstore_crypto::parse_certificate_info(cert_der.as_bytes()) {
        Ok(info) => info,
        Err(_) => return false,
    };

    let vk = match sigstore_crypto::VerificationKey::from_spki(
        &cert_info.public_key,
        cert_info.signing_scheme,
    ) {
        Ok(vk) => vk,
        Err(_) => return false,
    };

    let data_to_verify = pae(MANIFEST_PAYLOAD_TYPE, payload);

    signatures
        .iter()
        .any(|sig| vk.verify(&data_to_verify, &sig.signature).is_ok())
}

fn load_cert_from_file(cert_path: &Path) -> Result<DerCertificate> {
    let pem_data = std::fs::read_to_string(cert_path)
        .map_err(|e| Error::Signing(format!("Failed to read certificate file: {e}")))?;
    DerCertificate::from_pem(&pem_data)
        .map_err(|e| Error::Signing(format!("Failed to parse certificate PEM: {e}")))
}

/// Sign a DSSE envelope using an ephemeral ECDSA P-256 key and obtain a Fulcio certificate.
async fn sign_with_fulcio(payload: &[u8], oidc_token: &str) -> Result<(Envelope, DerCertificate)> {
    use crate::in_toto::dsse::pae;
    use sigstore_crypto::KeyPair;
    use sigstore_fulcio::FulcioClient;
    use sigstore_oidc::IdentityToken;
    use sigstore_types::SignatureBytes;

    let token = IdentityToken::from_jwt(oidc_token)
        .map_err(|e| Error::Signing(format!("Invalid OIDC token: {e}")))?;

    let key_pair = KeyPair::generate_ecdsa_p256()
        .map_err(|e| Error::Signing(format!("Failed to generate ephemeral key: {e}")))?;

    let fulcio = FulcioClient::public();
    let cert_response = fulcio
        .create_signing_certificate(&token, &key_pair)
        .await
        .map_err(|e| Error::Signing(format!("Fulcio certificate request failed: {e}")))?;

    let cert = cert_response
        .leaf_certificate()
        .map_err(|e| Error::Signing(format!("Failed to extract Fulcio certificate: {e}")))?;

    let payload_type = MANIFEST_PAYLOAD_TYPE.to_string();
    let data_to_sign = pae(&payload_type, payload);
    let sig: SignatureBytes = key_pair
        .sign(&data_to_sign)
        .map_err(|e| Error::Signing(format!("Ephemeral key signing failed: {e}")))?;

    let mut envelope = Envelope::new(&payload.to_vec(), payload_type);
    envelope.add_signature(sig.as_bytes().to_vec(), "".to_string())?;

    Ok((envelope, cert))
}
