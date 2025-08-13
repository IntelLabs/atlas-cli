# Common Functionality for Atlas Examples

The scripts in this directory implement common Atlas CLI commands used in demos
and examples. Different examples may select which scripts they import.

* [config.sh]: Common configuration variables, incl. default storage backend,
  signing key files, etc.
* [keys.sh](./keys.sh): Signing key generation and removal (requires `openssl`).
* [manifest_create.sh](./manifest_create.sh): Common manifest creation commands
  for models, datasets, software and evaluation, as well as linking functions.
* [manifest_verify.sh](./manifest_verify.sh): Common manifest verification
  commands.
* [manifest_utils.sh](./manifest_utils.sh): Common manifest utilies, incl.
  manifest display, export, and ID extraction.

**Note:** All scripts use variables defined in [config.sh], so we recommend
always including a `source` line for this script in any example or demo script.

[config.sh]: ./config.sh
