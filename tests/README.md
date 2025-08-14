# Helper Functions for Atlas CLI Tests

The scripts in this directory implement common Atlas CLI commands used tests.
Tests also commonly import [common utility scripts] from the `/examples`.

* [manifest_create.sh](./manifest_create.sh): Common manifest creation commands
  for models, datasets, software and evaluation, as well as linking functions.
* [manifest_verify.sh](./manifest_verify.sh): Common manifest verification
  commands.
* [manifest_utils.sh](./manifest_utils.sh): Common manifest utilies, incl.
  manifest display, export, and ID extraction.

**Note:** All scripts use variables defined in [config.sh], so we recommend
always including a `source` line for this script in any example or demo script.

[config.sh]: ../examples/common/config.sh
[common utility scripts]: ../examples/common/README.md
