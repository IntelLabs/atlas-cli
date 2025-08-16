use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum HashAlgorithmChoice {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgorithmChoice {
    pub fn to_cose_algorithm(&self) -> atlas_c2pa_lib::cose::HashAlgorithm {
        match self {
            HashAlgorithmChoice::Sha256 => atlas_c2pa_lib::cose::HashAlgorithm::Sha256,
            HashAlgorithmChoice::Sha384 => atlas_c2pa_lib::cose::HashAlgorithm::Sha384,
            HashAlgorithmChoice::Sha512 => atlas_c2pa_lib::cose::HashAlgorithm::Sha512,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum DatasetCommands {
    /// Create a new dataset manifest
    Create {
        #[arg(long = "paths", num_args = 1.., value_delimiter = ',')]
        paths: Vec<PathBuf>,

        /// Names for each ingredient (comma-separated)
        #[arg(long = "ingredient-names", num_args = 1.., value_delimiter = ',')]
        ingredient_names: Vec<String>,

        /// Dataset name
        #[arg(long = "name")]
        name: String,

        /// Author organization name
        #[arg(long = "author-org")]
        author_org: Option<String>,

        /// Author name
        #[arg(long = "author-name")]
        author_name: Option<String>,

        /// Optional description
        #[arg(long = "description")]
        description: Option<String>,

        /// Optional linked manifest IDs
        #[arg(long = "linked-manifests")]
        linked_manifests: Option<Vec<String>>,

        /// Path to private key file for signing (PEM format)
        #[arg(long = "key")]
        key: Option<PathBuf>,

        /// Hash algorithm to use for signing (default: sha384)
        #[arg(long = "hash-alg", value_enum, default_value = "sha384")]
        hash_alg: HashAlgorithmChoice,

        /// Only print manifest without storing
        #[arg(long = "print")]
        print: bool,

        /// Output format (json or cbor)
        #[arg(long = "format", default_value = "json")]
        format: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,

        /// Collect the underlying TDX attestation, if available
        #[arg(long = "with-tdx", default_value = "false")]
        with_tdx: bool,
    },
    /// List all dataset manifests
    List {
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    Verify {
        /// Manifest ID to verify
        #[arg(long = "id")]
        id: String,
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ModelCommands {
    /// Create a new signed model manifest compliant with OpenSSF Model Signing (OMS) specification
    Create {
        /// Paths to the model ingredient files
        #[arg(long = "paths", num_args = 1.., value_delimiter = ',')]
        paths: Vec<PathBuf>,

        /// Names for each ingredient (comma-separated)
        #[arg(long = "ingredient-names", num_args = 1.., value_delimiter = ',')]
        ingredient_names: Vec<String>,

        /// Model name
        #[arg(long = "name")]
        name: String,

        /// Author organization name
        #[arg(long = "author-org")]
        author_org: Option<String>,

        /// Author name
        #[arg(long = "author-name")]
        author_name: Option<String>,

        /// Optional description
        #[arg(long = "description")]
        description: Option<String>,

        /// Optional linked manifest IDs
        #[arg(long = "linked-manifests")]
        linked_manifests: Option<Vec<String>>,

        /// Path to private key file for signing (PEM format)
        #[arg(long = "key")]
        key: Option<PathBuf>,

        /// Hash algorithm to use for signing (default: sha384)
        #[arg(long = "hash-alg", value_enum, default_value = "sha384")]
        hash_alg: HashAlgorithmChoice,

        /// Only print manifest without storing
        #[arg(long = "print")]
        print: bool,

        /// Encoding (json or cbor)
        #[arg(long = "encoding", default_value = "json")]
        encoding: String,

        /// Format (standalone c2pa or oms)
        #[arg(long = "format", default_value = "standalone")]
        format: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,

        /// Collect the underlying CC attestation, if available
        #[arg(long = "with-tdx", default_value = "false")]
        with_tdx: bool,
    },
    /// List all model manifests
    List {
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    Verify {
        /// Manifest ID to verify
        #[arg(long = "id")]
        id: String,
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    LinkDataset {
        /// Model manifest ID
        #[arg(long = "model-id")]
        model_id: String,

        /// Dataset manifest ID
        #[arg(long = "dataset-id")]
        dataset_id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
}

#[derive(Subcommand)]
pub enum ManifestCommands {
    /// Link manifests together
    Link {
        /// Source manifest ID
        #[arg(short, long)]
        source: String,

        /// Target manifest ID
        #[arg(short, long)]
        target: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },

    /// Show manifest details
    Show {
        /// Manifest ID to show
        #[arg(short, long)]
        id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },

    /// Validate manifest cross-references
    Validate {
        /// Manifest ID to validate
        #[arg(short, long)]
        id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },

    /// Verify a specific link between two manifests
    VerifyLink {
        /// Source manifest ID
        #[arg(short, long)]
        source: String,

        /// Target manifest ID
        #[arg(short, long)]
        target: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    /// Export provenance graph information
    Export {
        /// Manifest ID to export provenance for
        #[arg(short, long)]
        id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,

        /// Output format (json or yaml)
        #[arg(long = "format", default_value = "json")]
        format: String,

        /// Output file path (defaults to stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,

        /// Max depth to traverse the provenance graph (default: 10)
        #[arg(long = "max-depth", default_value = "10")]
        max_depth: u32,
    },
}
/// Commands for evaluation results
#[derive(Subcommand)]
pub enum EvaluationCommands {
    /// Create a new evaluation result manifest
    Create {
        /// Path to evaluation results file
        #[arg(long = "path")]
        path: PathBuf,

        /// Evaluation name
        #[arg(long = "name")]
        name: String,

        /// Model ID that was evaluated
        #[arg(long = "model-id")]
        model_id: String,

        /// Evaluation dataset ID
        #[arg(long = "dataset-id")]
        dataset_id: String,

        /// Evaluation metrics (key=value pairs)
        #[arg(long = "metrics", num_args = 1.., value_delimiter = ',')]
        metrics: Vec<String>,

        /// Author organization name
        #[arg(long = "author-org")]
        author_org: Option<String>,

        /// Author name
        #[arg(long = "author-name")]
        author_name: Option<String>,

        /// Optional description
        #[arg(long = "description")]
        description: Option<String>,

        /// Path to private key file for signing (PEM format)
        #[arg(long = "key")]
        key: Option<PathBuf>,

        /// Hash algorithm to use for signing (default: sha384)
        #[arg(long = "hash-alg", value_enum, default_value = "sha384")]
        hash_alg: HashAlgorithmChoice,

        /// Only print manifest without storing
        #[arg(long = "print")]
        print: bool,

        /// Output format (json or cbor)
        #[arg(long = "format", default_value = "json")]
        format: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },

    /// List all evaluation results
    List {
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },

    /// Verify an evaluation result manifest
    Verify {
        /// Evaluation result manifest ID to verify
        #[arg(long = "id")]
        id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
}

/// CCAttestationCommands are mostly for debugging since CC attestations
/// will be collected directly during asset creation/modification
#[derive(Subcommand)]
pub enum CCAttestationCommands {
    /// Reads the CC attestation and display it
    Show,
    /// Extracts the VM launch measurement value from the CC attestation
    GetLaunchMeasurement,
    /// Verify VM launch measurement against a launch endorsement
    VerifyLaunch {
        /// VM host platform (determines download command and format)
        #[arg(long = "host-platform", default_value = "gcp-tdx")]
        host_platform: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SoftwareCommands {
    /// Create a new software component manifest
    Create {
        #[arg(long = "paths", num_args = 1.., value_delimiter = ',')]
        paths: Vec<PathBuf>,

        /// Names for each component (comma-separated)
        #[arg(long = "ingredient-names", num_args = 1.., value_delimiter = ',')]
        ingredient_names: Vec<String>,

        /// Software name
        #[arg(long = "name")]
        name: String,

        /// Software type (script, container, VM, etc.)
        #[arg(long = "software-type")]
        software_type: String,

        /// Software version
        #[arg(long = "version")]
        version: Option<String>,

        /// Author organization name
        #[arg(long = "author-org")]
        author_org: Option<String>,

        /// Author name
        #[arg(long = "author-name")]
        author_name: Option<String>,

        /// Optional description
        #[arg(long = "description")]
        description: Option<String>,

        /// Optional linked manifest IDs
        #[arg(long = "linked-manifests")]
        linked_manifests: Option<Vec<String>>,

        /// Path to private key file for signing (PEM format)
        #[arg(long = "key")]
        key: Option<PathBuf>,

        /// Hash algorithm to use for signing (default: sha384)
        #[arg(long = "hash-alg", value_enum, default_value = "sha384")]
        hash_alg: HashAlgorithmChoice,

        /// Only print manifest without storing
        #[arg(long = "print")]
        print: bool,

        /// Output format (json or cbor)
        #[arg(long = "format", default_value = "json")]
        format: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,

        /// Collect the underlying CC attestation, if available
        #[arg(long = "with-tdx", default_value = "false")]
        with_tdx: bool,
    },
    /// List all software component manifests
    List {
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    /// Verify a software component manifest
    Verify {
        /// Manifest ID to verify
        #[arg(long = "id")]
        id: String,
        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    /// Link software to a model
    LinkModel {
        /// Software manifest ID
        #[arg(long = "software-id")]
        software_id: String,

        /// Model manifest ID
        #[arg(long = "model-id")]
        model_id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
    /// Link software to a dataset
    LinkDataset {
        /// Software manifest ID
        #[arg(long = "software-id")]
        software_id: String,

        /// Dataset manifest ID
        #[arg(long = "dataset-id")]
        dataset_id: String,

        /// Storage backend (local or rekor)
        #[arg(long = "storage-type", default_value = "database")]
        storage_type: Box<String>,

        /// Storage URL
        #[arg(long = "storage-url", default_value = "http://localhost:8080")]
        storage_url: Box<String>,
    },
}
