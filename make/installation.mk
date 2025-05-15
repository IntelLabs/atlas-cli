.PHONY: release install help-install

# Build in release mode
release:
	$(CARGO) build --release $(BUILD_FLAGS)

# Install the binary
install: release
	@echo "Installing $(BINARY_NAME) to /usr/local/bin..."
	cp $(RELEASE_TARGET_DIR)/$(BINARY_NAME) /usr/local/bin/
	@echo "Installation complete. You can run '$(BINARY_NAME)' from any directory."

# Help text for installation targets
help-install:
	@echo "Installation targets for atlas-cli:"
	@echo "=================================================================================="
	@echo "  make release      - Build the project in release mode"
	@echo "  make install      - Install the binary to /usr/local/bin"
	@echo ""
	@echo "Configuration variables:"
	@echo "  BINARY_NAME       - Name of the binary (default: $(BINARY_NAME))"
	@echo "  RELEASE_TARGET_DIR - Release build directory (default: $(RELEASE_TARGET_DIR))"
	@echo "  BUILD_FLAGS       - Additional flags for cargo build (default: $(BUILD_FLAGS))"