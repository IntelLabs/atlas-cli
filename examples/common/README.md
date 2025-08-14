# Common Utilities for Atlas Examples

The scripts in this directory implement utility functions used in demos and
examples. Different examples may select which scripts they import.

* [config.sh]: Common configuration variables, incl. default storage backend,
  signing key files, etc.
* [keys.sh](./keys.sh): Signing key generation and removal (requires `openssl`).
* [utils.sh](./utils.sh): Common utilies, incl. manifest ID extraction, action
  if files do not exist, and waiting for user.

**Note:** All scripts use variables defined in [config.sh], so we recommend
always including a `source` line for this script in any example or demo script.

[config.sh]: ./config.sh
