use serde::{Deserialize, Serialize};
use serde_json::json;

use tdx_workload_attestation::error::Result;
use tdx_workload_attestation::provider::AttestationProvider;

pub struct MockAttestationProvider {
    platform: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MockReport {
    report_type: String,
    platform: String,
    timestamp: String,
    status: String,
    version: String,
    message: String,
}

impl MockAttestationProvider {
    pub fn new(platform: &str) -> Self {
        Self {
            platform: platform.to_string(),
        }
    }
}

impl AttestationProvider for MockAttestationProvider {
    fn get_attestation_report(&self) -> Result<String> {
        // Create a mock attestation report with platform info
        let mock_report = json!({
            "report_type": "mock_attestation",
            "platform": self.platform,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": "1.0",
            "status": "simulated",
            "message": "This is a mock attestation report for non-Linux or unsupported platforms"
        });

        // Serialize to JSON string
        Ok(serde_json::to_string_pretty(&mock_report).unwrap_or_else(|_| "{}".to_string()))
    }

    fn get_launch_measurement(&self) -> Result<[u8; 48]> {
        Ok([0; 48])
    }
}
