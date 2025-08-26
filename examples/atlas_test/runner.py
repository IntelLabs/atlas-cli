"""CLI runner for Atlas Test Framework"""

import click
import sys
import logging
from pathlib import Path
from .framework import AtlasTestFramework
from .utils import setup_logging

logger = logging.getLogger(__name__)


@click.command()
@click.argument("config_file", type=click.Path(exists=True))
@click.option("--dry-run", is_flag=True, help="Print commands without executing")
@click.option("--interactive", is_flag=True, help="Pause between steps")
@click.option("--continue-on-error", is_flag=True, help="Continue on errors")
@click.option("--output-dir", help="Output directory for logs and scripts")
@click.option("--verbose", is_flag=True, help="Verbose output")
def main(config_file, dry_run, interactive, continue_on_error, output_dir, verbose):
    """Run Atlas CLI test framework with specified configuration"""

    # Setup logging
    setup_logging(verbose)

    try:
        # Initialize framework
        framework = AtlasTestFramework(config_file)

        # Override settings from command line
        if dry_run:
            framework.config.setdefault("environment", {})["dry_run"] = True
        if interactive:
            framework.config.setdefault("environment", {})["interactive"] = True
        if continue_on_error:
            framework.config.setdefault("environment", {})["continue_on_error"] = True
        if output_dir:
            framework.config.setdefault("environment", {})["output_dir"] = output_dir
            # Update recorder output dir
            framework.command_recorder.output_dir = output_dir
            framework.command_recorder.log_file = Path(output_dir) / "commands.log"
            framework.command_recorder.script_file = Path(output_dir) / "reproduce.sh"

        # Run workflow
        logger.info("🚀 Starting Atlas Test Framework")
        framework.setup()
        framework.execute()

        logger.info("\\n✅ WORKFLOW COMPLETED SUCCESSFULLY")

    except KeyboardInterrupt:
        logger.warning("\\n⚠️  Interrupted by user")
        sys.exit(1)

    except Exception as e:
        logger.error(f"\\n❌ Error: {e}")
        if verbose:
            import traceback

            traceback.print_exc()
        sys.exit(1)

    finally:
        if "framework" in locals():
            framework.teardown()


if __name__ == "__main__":
    main()
