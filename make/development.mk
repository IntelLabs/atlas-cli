.PHONY: fmt lint doc check test-verbose watch-test dev-deps generate-keys help-dev version

# Format code
fmt:
	$(CARGO) fmt

# Run clippy for linting
lint:
	$(CARGO) clippy -- -D warnings

# Generate documentation
doc:
	$(CARGO) doc --no-deps

# Comprehensive check before commit
check: fmt lint test
	@echo "All checks passed!"

# Run tests with output
test-verbose:
	$(CARGO) test -- --nocapture $(TEST_FLAGS)

# Run tests with output (`with-tdx` feature enabled)
test-with-tdx:
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' $(CARGO) test --tests --features with-tdx $(TEST_FLAGS)

# Watch for changes and run tests
watch-test:
	cargo watch -x test

# Install development dependencies
dev-deps:
	cargo install cargo-watch
	cargo install cargo-edit

# Generate test keys for signing
generate-keys:
	openssl genpkey -algorithm RSA -out private.pem -pkeyopt rsa_keygen_bits:2048
	openssl rsa -pubout -in private.pem -out public.pem

# Display version information
version:
	@echo "atlas-cli version: $(VERSION)"
	@$(CARGO) --version
	@rustc --version

# Help text for development targets
help-dev:
	@echo "Development targets for atlas-cli:"
	@echo "=================================================================================="
	@echo "  make fmt          - Format code using cargo fmt"
	@echo "  make lint         - Run clippy linter"
	@echo "  make doc          - Generate documentation"
	@echo "  make check        - Run format, lint, and tests"
	@echo "  make test-verbose - Run tests with output"
	@echo "  make watch-test   - Watch for changes and run tests"
	@echo "  make dev-deps     - Install development dependencies"
	@echo "  make generate-keys - Generate RSA keys for signing"
	@echo "  make version      - Display version information"
