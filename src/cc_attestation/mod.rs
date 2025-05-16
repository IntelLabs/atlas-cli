use crate::error::{Error, Result};

pub mod mock;
use mock::MockAttestationProvider;

use tdx_workload_attestation::get_platform_name;
use tdx_workload_attestation::provider::AttestationProvider;

#[cfg(feature = "with-tdx")]
use tdx_workload_attestation::gcp::GcpTdxHost;
#[cfg(feature = "with-tdx")]
use tdx_workload_attestation::host::TeeHost;
#[cfg(feature = "with-tdx")]
use tdx_workload_attestation::tdx::LinuxTdxProvider;

pub fn get_report(show: bool) -> Result<String> {
    // Select the appropriate provider based on platform and current OS
    let platform = get_platform_name().map_err(|e| Error::CCAttestationError(e.to_string()))?;

    let provider: Box<dyn AttestationProvider> = match platform.as_str() {
        #[cfg(feature = "with-tdx")]
        "tdx-linux" => Box::new(LinuxTdxProvider::new()),
        _ => Box::new(MockAttestationProvider::new(&platform)), // Use mock for non-Linux
    };

    // Get the attestation report from the provider
    let report = provider
        .get_attestation_report()
        .map_err(|e| Error::CCAttestationError(e.to_string()))?;

    if show {
        println!("Got report: {}", &report);
    }

    Ok(report)
}

pub fn get_launch_measurement() -> Result<[u8; 48]> {
    // Select the appropriate provider based on platform and current OS
    let platform = get_platform_name().map_err(|e| Error::CCAttestationError(e.to_string()))?;

    let provider: Box<dyn AttestationProvider> = match platform.as_str() {
        #[cfg(feature = "with-tdx")]
        "tdx-linux" => Box::new(LinuxTdxProvider::new()),
        _ => Box::new(MockAttestationProvider::new(&platform)), // Use mock for non-Linux, non-CC
    };

    // Get the measurement from the provider
    let measurement = provider
        .get_launch_measurement()
        .map_err(|e| Error::CCAttestationError(e.to_string()))?;

    Ok(measurement)
}

pub fn verify_launch_endorsement(host_platform: &str) -> Result<bool> {
    // Get the launch endorsement from the specific host, if possible
    match host_platform {
        #[cfg(feature = "with-tdx")]
        "gcp-tdx" => {
            // Get the launch measurement for the current platform
            let measurement =
                get_launch_measurement().map_err(|e| Error::CCAttestationError(e.to_string()))?;
            let gcp_host = GcpTdxHost::new(&measurement);
            Ok(gcp_host
                .verify_launch_endorsement()
                .map_err(|e| Error::CCAttestationError(e.to_string()))?)
        }
        _ => Err(Error::CCAttestationError(format!(
            "Launch endorsement verification not supported for platform {}",
            host_platform
        ))),
    }
}
