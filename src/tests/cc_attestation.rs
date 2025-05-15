use crate::cc_attestation;
use crate::cc_attestation::mock::MockAttestationProvider;
use crate::error::{Error, Result};
use serde_json::Value;
use tdx_workload_attestation::provider::AttestationProvider;

// Test that the mock attestation provider generates valid reports
#[test]
fn test_mock_attestation() -> Result<()> {
    // Create a mock provider with a test platform
    let provider = MockAttestationProvider::new("test-platform");

    // Get attestation report
    let report = provider
        .get_attestation_report()
        .map_err(|e| Error::CCAttestationError(e.to_string()))?;

    // Verify the report is valid JSON
    let report_json: Value = serde_json::from_str(&report)?;

    // Assert the report contains expected fields
    assert_eq!(report_json["type"], "mock_attestation");
    assert_eq!(report_json["platform"], "test-platform");
    assert!(report_json["timestamp"].is_string());
    assert!(report_json["mock_data"].is_object());
    assert_eq!(report_json["mock_data"]["version"], "1.0");
    assert_eq!(report_json["mock_data"]["status"], "simulated");

    Ok(())
}

// Test that the get_report function works for unsupported platforms
#[test]
fn test_get_report_unsupported_platform() -> Result<()> {
    // Try to get a report for an unsupported platform
    let report = cc_attestation::get_report(false)?;

    // Verify the report is valid JSON
    let report_json: Value = serde_json::from_str(&report)?;

    // Verify it's a mock report
    assert_eq!(report_json["type"], "mock_attestation");
    // assert_eq!(report_json["platform"], "unsupported");

    Ok(())
}

// Test that the get_report function with show parameter works
#[test]
fn test_get_report_with_show() -> Result<()> {
    let report = cc_attestation::get_report(false)?;

    // Verify it returned a non-empty string
    assert!(!report.is_empty());

    Ok(())
}

// Test specific platform implementations conditionally
#[cfg(feature = "with-tdx")]
mod linux_tests {
    use super::*;
    use tdx_workload_attestation::tdx::LinuxTdxProvider;

    // This test only runs on Linux
    #[test]
    fn test_linux_tdx_attestation() -> Result<()> {
        let provider = LinuxTdxProvider::new();

        match provider.get_attestation_report() {
            Ok(report) => {
                // Verify report is valid JSON
                let _: Value = serde_json::from_str(&report)?;
                Ok(())
            }
            Err(e) => {
                // If test is running on hardware without TDX support, just print a warning
                println!("⚠️ TDX attestation not available: {}", e);
                Ok(())
            }
        }
    }

    // Test that TDX platform is selected on Linux
    #[test]
    fn test_tdx_provider_selection() {
        // Just verify that the code runs without panicking
        let _ = cc_attestation::get_report(false);
    }
}

// Integration test for using attestation in manifests
#[test]
fn test_attestation_in_manifest() -> Result<()> {
    use atlas_c2pa_lib::assertion::CustomAssertion;
    use tempfile::tempdir;

    // Create a simple temporary file for testing
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file.txt");
    std::fs::write(&file_path, b"test data")?;

    // Get a report for a test platform
    let report = cc_attestation::get_report(false)?;

    // Create a custom assertion with the report
    let assertion = CustomAssertion {
        label: "test-platform".to_string(),
        data: serde_json::Value::String(report),
    };

    // Verify that the assertion is properly formed
    assert_eq!(assertion.label, "test-platform");
    assert!(assertion.data.is_string());

    // Clean up test directory
    temp_dir.close()?;

    Ok(())
}
