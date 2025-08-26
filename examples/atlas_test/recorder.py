"""Command recording and reproduction functionality"""

import os
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime
import subprocess


class CommandRecorder:
    """Records all executed commands for replay and debugging"""

    def __init__(self, output_dir: Optional[str] = None):
        self.commands: List[Dict[str, Any]] = []
        self.output_dir = output_dir
        self.log_file = None
        self.script_file = None

        if output_dir:
            Path(output_dir).mkdir(parents=True, exist_ok=True)
            self.log_file = Path(output_dir) / "commands.log"
            self.script_file = Path(output_dir) / "reproduce.sh"
            # Clear log file at start
            if self.log_file.exists():
                self.log_file.unlink()

    def record(
        self,
        command: str,
        result: Optional[subprocess.CompletedProcess] = None,
        success: bool = True,
        step_name: Optional[str] = None,
        output_id: Optional[str] = None,
    ):
        """Record a command execution"""
        entry = {
            "timestamp": datetime.now().isoformat(),
            "step": step_name or "Unknown",
            "command": command,
            "success": success,
            "output_id": output_id,
            "return_code": result.returncode if result else 0,
            "stdout": result.stdout if result else None,
            "stderr": result.stderr if result else None,
        }
        self.commands.append(entry)

        # Write to log file immediately
        if self.log_file:
            with open(self.log_file, "a") as f:
                f.write(f"\\n{'='*80}\\n")
                f.write(f"[{entry['timestamp']}] Step: {entry['step']}\\n")
                f.write(f"$ {command}\\n")
                f.write(f"Return Code: {entry['return_code']}\\n")

                if entry["stdout"] and entry["stdout"].strip():
                    f.write(f"\\nSTDOUT:\\n{entry['stdout']}\\n")

                if entry["stderr"] and entry["stderr"].strip():
                    f.write(f"\\nSTDERR:\\n{entry['stderr']}\\n")

                if output_id:
                    f.write(f"\\nGenerated ID: {output_id}\\n")

    def export_script(self, variables: Optional[Dict[str, str]] = None):
        """Export all commands as an executable shell script"""
        if not self.script_file:
            return

        with open(self.script_file, "w") as f:
            f.write("#!/bin/bash\\n")
            f.write("# Atlas CLI Test Reproduction Script\\n")
            f.write(f"# Generated: {datetime.now().isoformat()}\\n")
            f.write(
                "# This script reproduces all Atlas CLI commands from the test run\\n\\n"
            )

            # Error handling
            f.write("set -e  # Exit on error\\n")
            f.write("set -u  # Exit on undefined variable\\n\\n")

            # Color output
            f.write("# Colors for output\\n")
            f.write('RED="\\\\033[0;31m"\\n')
            f.write('GREEN="\\\\033[0;32m"\\n')
            f.write('YELLOW="\\\\033[1;33m"\\n')
            f.write('NC="\\\\033[0m"\\n\\n')

            # Check Atlas CLI
            f.write("# Check Atlas CLI is available\\n")
            f.write("if ! command -v atlas-cli &> /dev/null; then\\n")
            f.write('    echo -e "${RED}Error: atlas-cli not found in PATH${NC}"\\n')
            f.write('    echo "Please install Atlas CLI first"\\n')
            f.write("    exit 1\\n")
            f.write("fi\\n\\n")

            f.write(
                'echo -e "${GREEN}Starting Atlas CLI test reproduction...${NC}"\\n\\n'
            )

            # Add variables if provided
            if variables:
                f.write("# Manifest IDs from original test run\\n")
                for key, value in variables.items():
                    f.write(f'# {key}="{value}"\\n')
                f.write("\\n")

            # Function to extract manifest ID
            f.write("# Helper function to extract manifest ID from output\\n")
            f.write("extract_manifest_id() {\\n")
            f.write(
                '    grep -oE "Manifest stored successfully with ID: [^ ]+" | cut -d" " -f6 || \\\\\\n'
            )
            f.write('    grep -oE "ID: [^ ]+" | cut -d" " -f2 || \\\\\\n')
            f.write('    echo "unknown"\\n')
            f.write("}\\n\\n")

            # Add commands
            current_step = None
            for i, cmd in enumerate(self.commands):
                if cmd["step"] != current_step:
                    current_step = cmd["step"]
                    f.write(f'\\n# {"="*60}\\n')
                    f.write(f"# Step: {current_step}\\n")
                    f.write(f'# {"="*60}\\n')
                    f.write(
                        f'echo -e "\\\\n${{GREEN}}▶ Executing: {current_step}${{NC}}"\\n\\n'
                    )

                # Add the command
                f.write(f"# Command {i+1}\\n")
                f.write(f"{cmd['command']}\\n")

                # Add error checking
                f.write("if [ $? -ne 0 ]; then\\n")
                f.write(f'    echo -e "${{RED}}✗ Failed: {current_step}${{NC}}"\\n')
                f.write("    exit 1\\n")
                f.write("else\\n")
                f.write(
                    f'    echo -e "${{GREEN}}✓ Completed: {current_step}${{NC}}"\\n'
                )
                f.write("fi\\n\\n")

            f.write(
                'echo -e "\\\\n${GREEN}✅ All commands executed successfully!${NC}"\\n'
            )

        # Make script executable
        os.chmod(self.script_file, 0o755)
        print(f"📝 Reproduction script saved to: {self.script_file}")

    def show_summary(self):
        """Display execution summary"""
        total = len(self.commands)
        successful = sum(1 for c in self.commands if c["success"])
        failed = total - successful

        print("\\n" + "=" * 80)
        print("📊 EXECUTION SUMMARY")
        print("=" * 80)
        print(f"Total Commands: {total}")
        print(f"✅ Successful: {successful}")

        if failed > 0:
            print(f"❌ Failed: {failed}")
            print("\\nFailed Commands:")
            for cmd in self.commands:
                if not cmd["success"]:
                    print(f"  - [{cmd['step']}] {cmd['command'][:100]}...")

        if self.log_file:
            print(f"\\n📄 Full log: {self.log_file}")
        if self.script_file:
            print(f"📜 Reproduction script: {self.script_file}")

        print("=" * 80)
