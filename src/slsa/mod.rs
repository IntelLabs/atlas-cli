//! # SLSA (Supply-chain Levels for Software Artifacts) Implementation
//!
//! This module provides functionality for generating SLSA (Supply-chain Levels for Software Artifacts)
//! v1 build provenance attestations. SLSA is a security framework that helps protect the integrity of
//! software artifacts throughout the development and deployment pipeline.
//!
//! ## Overview
//!
//! SLSA provenance provides a cryptographically verifiable record of:
//! - What software was built
//! - How it was built (build process, tools, environment)
//! - Who built it (builder identity and authentication)
//! - When it was built (timestamps and metadata)
//!
//! ## SLSA Build Levels
//!
//! This implementation supports SLSA Build provenance with varying levels of security:
//! - **Build L1**: Basic provenance generation with minimal requirements
//! - **Build L2**: Enhanced provenance with hosted build service requirements
//! - **Build L3**: High-confidence provenance with additional security controls
//!
//! ## Key Components
//!
//! - [`cli`] - An Atlas CLI specific generator for SLSA Build Provenance
//! - [`generators`] - Core functions for creating SLSA provenance structures
//! - [`BUILD_PROVENANCE_PREDICATE_TYPE_V1`] - Standard SLSA v1 predicate type URI
//!
//! ## Examples
//!
//! ### Basic Build Provenance Generation
//!
//! ```no_run
//! use atlas_cli::slsa::cli::generate_build_provenance;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::PathBuf;
//!
//! // Generate SLSA build provenance for compiled artifacts
//! generate_build_provenance(
//!     vec![PathBuf::from("src/main.rs")],           // input files
//!     PathBuf::from("Makefile"),                    // build script
//!     vec![PathBuf::from("target/release/myapp")],  // output artifacts  
//!     Some(PathBuf::from("signing_key.pem")),       // signing key
//!     HashAlgorithm::Sha384,                        // hash algorithm
//!     "json".to_string(),                           // output format
//!     true,                                         // print to stdout
//!     None,                                         // storage backend
//!     false,                                        // TDX support
//! ).unwrap();
//! ```
pub mod cli;
pub mod generators;

/// The standard SLSA v1 build provenance in-toto predicate type URI.
///
/// This constant defines the official predicate type identifier for SLSA build provenance
/// attestations according to the SLSA v1 specification. It is used in in-toto Statements
/// to indicate that the predicate contains SLSA build provenance information.
///
/// ```
/// use atlas_cli::slsa::BUILD_PROVENANCE_PREDICATE_TYPE_V1;
///
/// // Used in attestation generation
/// let predicate_type = BUILD_PROVENANCE_PREDICATE_TYPE_V1;
/// assert_eq!(predicate_type, "https://slsa.dev/provenance/v1");
/// ```
pub const BUILD_PROVENANCE_PREDICATE_TYPE_V1: &str = "https://slsa.dev/provenance/v1";
