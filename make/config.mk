# Basic configuration
CARGO := cargo
BINARY_NAME := atlas-cli
TARGET_DIR := target/debug
RELEASE_TARGET_DIR := target/release

# Optional configuration
FEATURES ?=
BUILD_FLAGS ?=
TEST_FLAGS ?=

# OS-specific configuration
ifeq ($(OS),Windows_NT)
    # Windows-specific configuration
    DETECTED_OS := Windows
    # Use PowerShell commands for Windows
    MKDIR_CMD := powershell -Command "function mkdir-p { param([string]$$path); if (!(Test-Path $$path)) { New-Item -ItemType Directory -Path $$path -Force | Out-Null } }"
    TOUCH_CMD := powershell -Command "function touch { param([string]$$path); if (!(Test-Path $$path)) { New-Item -ItemType File -Path $$path | Out-Null } else { (Get-Item $$path).LastWriteTime = Get-Date } }"
    RM_CMD := powershell -Command "Remove-Item -Path"
else
    # Unix-like OS configuration (Linux, macOS)
    DETECTED_OS := $(shell uname -s)
    MKDIR_CMD := mkdir -p
    TOUCH_CMD := touch
    RM_CMD := rm -f
endif

# Directories
EXAMPLES_DIR := examples
EXAMPLES_MODELS_DIR := $(EXAMPLES_DIR)/models
EXAMPLES_DATA_DIR := $(EXAMPLES_DIR)/data
EXAMPLES_DATA_TRAIN_DIR := $(EXAMPLES_DATA_DIR)/train
EXAMPLES_DATA_TEST_DIR := $(EXAMPLES_DATA_DIR)/test
EXAMPLES_DATA_VALIDATION_DIR := $(EXAMPLES_DATA_DIR)/validation
EXAMPLES_RESULTS_DIR := $(EXAMPLES_DIR)/results

# Storage configuration
DEFAULT_STORAGE_TYPE := database
DEFAULT_STORAGE_URL := http://localhost:8080
DEFAULT_FILESYSTEM_PATH := ./storage

# Version information < extract from Cargo.toml
VERSION := $(shell grep -m1 'version = ' Cargo.toml 2>/dev/null | cut -d'"' -f2 2>/dev/null || echo "unknown")