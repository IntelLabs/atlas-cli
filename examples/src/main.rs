use atlas_test::{
    AtlasTestFramework,
    config::ConfigOverrides,
    utils::{print_error, setup_logging},
};
use clap::Parser;
use colored::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "atlas-test",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = "Test framework for Atlas CLI provenance and integrity testing",
    long_about = None
)]
struct Args {
    /// Configuration file (YAML)
    #[clap(value_name = "CONFIG_FILE")]
    config_file: PathBuf,

    /// Print commands without executing
    #[clap(short = 'd', long)]
    dry_run: bool,

    /// Pause between steps for interactive mode
    #[clap(short = 'i', long)]
    interactive: bool,

    /// Continue on errors instead of stopping
    #[clap(short = 'c', long)]
    continue_on_error: bool,

    /// Output directory for logs and scripts
    #[clap(short = 'o', long, value_name = "DIR")]
    output_dir: Option<String>,

    /// Enable verbose output
    #[clap(short = 'v', long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.verbose);

    // Print header
    println!("{}", "═".repeat(80).bright_blue());
    println!("{}", "  Atlas Test Framework  ".bold().bright_white());
    println!(
        "{}",
        format!("  Version {}  ", env!("CARGO_PKG_VERSION")).dimmed()
    );
    println!("{}", "═".repeat(80).bright_blue());

    // Run the framework
    match run_framework(args) {
        Ok(_) => {
            println!("\n{}", "✅ WORKFLOW COMPLETED SUCCESSFULLY".green().bold());
            std::process::exit(0);
        }
        Err(e) => {
            print_error(&format!("Error: {}", e));
            std::process::exit(1);
        }
    }
}

fn run_framework(args: Args) -> atlas_test::Result<()> {
    // Check if config file exists
    if !args.config_file.exists() {
        return Err(atlas_test::error::AtlasError::ConfigError(format!(
            "Configuration file not found: {}",
            args.config_file.display()
        )));
    }

    // Initialize framework
    let mut framework = AtlasTestFramework::new(&args.config_file)?;

    // Apply command-line overrides
    let overrides = ConfigOverrides {
        dry_run: if args.dry_run { Some(true) } else { None },
        interactive: if args.interactive { Some(true) } else { None },
        continue_on_error: if args.continue_on_error {
            Some(true)
        } else {
            None
        },
        output_dir: args.output_dir,
    };
    framework.apply_overrides(overrides);

    // Setup environment
    framework.setup()?;

    // Execute workflow
    let result = framework.execute();

    // Always run teardown, even on error
    let teardown_result = framework.teardown();

    // Check for errors
    result?;
    teardown_result?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = vec![
            "atlas-test",
            "config.yaml",
            "--dry-run",
            "--interactive",
            "--verbose",
        ];

        let parsed = Args::try_parse_from(args);
        assert!(parsed.is_ok());

        let args = parsed.unwrap();
        assert_eq!(args.config_file, PathBuf::from("config.yaml"));
        assert!(args.dry_run);
        assert!(args.interactive);
        assert!(args.verbose);
    }
}
