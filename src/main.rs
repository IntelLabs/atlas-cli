use atlas_cli::{
    cli::{
        self,
        commands::{
            CCAttestationCommands, DatasetCommands, EvaluationCommands, ManifestCommands,
            ModelCommands, PipelineCommands, SoftwareCommands,
        },
    },
    error::Result,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dataset-related commands
    Dataset {
        #[command(subcommand)]
        command: DatasetCommands,
    },
    /// Model-related commands
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },
    /// Software component commands
    Software {
        #[command(subcommand)]
        command: SoftwareCommands,
    },
    /// Manifest management commands
    Manifest {
        #[command(subcommand)]
        command: ManifestCommands,
    },
    /// Evaluation-related commands
    Evaluation {
        #[command(subcommand)]
        command: EvaluationCommands,
    },
    /// Pipeline-related commands
    Pipeline {
        #[command(subcommand)]
        command: PipelineCommands,
    },
    /// CC Attestation-related commands
    CCAttestation {
        #[command(subcommand)]
        command: CCAttestationCommands,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    atlas_cli::init_logging()?;

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle commands
    let result = match cli.command {
        Commands::Dataset { command } => cli::handlers::handle_dataset_command(command),
        Commands::Model { command } => cli::handlers::handle_model_command(command),
        Commands::Software { command } => cli::handlers::handle_software_command(command),

        Commands::Manifest { command } => cli::handlers::handle_manifest_command(command),
        Commands::Evaluation { command } => cli::handlers::handle_evaluation_command(command),
        Commands::Pipeline { command } => cli::handlers::handle_pipeline_command(command),
        Commands::CCAttestation { command } => {
            cli::handlers::handle_cc_attestation_command(command)
        }
    };

    // Format and display any errors
    if let Err(ref e) = result {
        eprintln!("{}", cli::format_error(e));
    }

    result
}
