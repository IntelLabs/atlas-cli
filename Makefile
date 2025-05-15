# Include configuration
include make/config.mk

# Def phony targets
.PHONY: all build release test clean help examples dev install

# Default target
all: build

# Basic targets
build:
	$(CARGO) build $(BUILD_FLAGS)

test:
	$(CARGO) test $(TEST_FLAGS)

clean:
	$(CARGO) clean
	rm -f private.pem public.pem

# Include specialized makefiles
include make/examples.mk
include make/development.mk
include make/installation.mk

# Help text
help:
	@echo "Atlas CLI - Makefile Help"
	@echo "=================================================================================="
	@echo "Main targets:"
	@echo "  make              - Build the project in debug mode"
	@echo "  make release      - Build the project in release mode"
	@echo "  make test         - Run tests"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make help         - Show this help text"
	@echo ""
	@echo "Development targets (see make help-dev):"
	@echo "  make fmt          - Format code"
	@echo "  make lint         - Run clippy linter"
	@echo "  make doc          - Generate documentation"
	@echo "  make check        - Run format, lint, and tests"
	@echo ""
	@echo "Installation targets (see make help-install):"
	@echo "  make install      - Install the binary to /usr/local/bin"
	@echo ""
	@echo "Example targets (see make help-examples):"
	@echo "  make examples     - Run all examples"
	@echo ""
	@echo "For more specific help on a category of targets, use:"
	@echo "  make help-dev     - Show development-related targets"
	@echo "  make help-install - Show installation-related targets"
	@echo "  make help-examples - Show example-related targets"