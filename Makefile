SHELL := /bin/sh

NPM ?= npm
CARGO ?= cargo

APP_NAME := porthole
TAURI_DIR := src-tauri
DIST_DIR := dist
BIN_DIR := bin

ifeq ($(OS),Windows_NT)
EXE_SUFFIX := .exe
HOST_GOST_BIN := $(TAURI_DIR)/binaries/gost-x86_64-pc-windows-msvc.exe
WINDOWS_TARGET ?= x86_64-pc-windows-msvc
WINDOWS_LINKER :=
WINDOWS_RC :=
WINDOWS_BUILD_GOST_BIN := $(TAURI_DIR)/binaries/gost-x86_64-pc-windows-msvc.exe
else
EXE_SUFFIX :=
HOST_GOST_BIN := $(TAURI_DIR)/binaries/gost-x86_64-unknown-linux-gnu
WINDOWS_TARGET ?= x86_64-pc-windows-gnu
WINDOWS_LINKER := CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
WINDOWS_RC := RC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-windres
WINDOWS_BUILD_GOST_BIN := $(TAURI_DIR)/binaries/gost-x86_64-pc-windows-gnu.exe
endif

RELEASE_BIN := $(TAURI_DIR)/target/release/$(APP_NAME)$(EXE_SUFFIX)
OUTPUT_BIN := $(BIN_DIR)/$(APP_NAME)$(EXE_SUFFIX)
WINDOWS_RELEASE_BIN := $(TAURI_DIR)/target/$(WINDOWS_TARGET)/release/$(APP_NAME).exe
WINDOWS_WEBVIEW2_LOADER := $(TAURI_DIR)/target/$(WINDOWS_TARGET)/release/WebView2Loader.dll
WINDOWS_OUTPUT_BIN := $(BIN_DIR)/$(APP_NAME).exe
WINDOWS_GOST_BIN := $(TAURI_DIR)/binaries/gost-x86_64-pc-windows-msvc.exe
WINDOWS_OUTPUT_GOST := $(BIN_DIR)/gost-x86_64-pc-windows-msvc.exe
WINDOWS_OUTPUT_GOST_ALIAS := $(BIN_DIR)/gost.exe
WINDOWS_OUTPUT_WEBVIEW2_LOADER := $(BIN_DIR)/WebView2Loader.dll

.PHONY: help install test build-frontend build-exe build build-win clean

help:
	@echo "Available targets:"
	@echo "  make install        Install frontend dependencies"
	@echo "  make test           Run frontend and Rust tests"
	@echo "  make build-frontend Build Vite frontend assets into $(DIST_DIR)"
	@echo "  make build-exe      Build the release executable at $(OUTPUT_BIN)"
	@echo "  make build          Alias of build-exe"
	@echo "  make build-win      Build Windows runtime files into $(BIN_DIR) using $(WINDOWS_TARGET)"
	@echo "  make clean          Remove frontend and Rust build outputs"

install:
	$(NPM) install

test:
	$(NPM) run test
	cd $(TAURI_DIR) && $(CARGO) test

build-frontend:
	$(NPM) run build

build-exe: build-frontend
	cd $(TAURI_DIR) && $(CARGO) build --release
	mkdir -p $(BIN_DIR)
	cp $(RELEASE_BIN) $(OUTPUT_BIN)
	test ! -f $(HOST_GOST_BIN) || cp $(HOST_GOST_BIN) $(BIN_DIR)/
	@echo "Executable built at $(OUTPUT_BIN)"

build: build-exe

build-win: build-frontend
	mkdir -p $(TAURI_DIR)/binaries
	test -f $(WINDOWS_BUILD_GOST_BIN) || cp $(WINDOWS_GOST_BIN) $(WINDOWS_BUILD_GOST_BIN)
	cd $(TAURI_DIR) && $(WINDOWS_LINKER) $(WINDOWS_RC) $(CARGO) build --release --target $(WINDOWS_TARGET)
	mkdir -p $(BIN_DIR)
	cp $(WINDOWS_RELEASE_BIN) $(WINDOWS_OUTPUT_BIN)
	cp $(WINDOWS_GOST_BIN) $(WINDOWS_OUTPUT_GOST)
	cp $(WINDOWS_GOST_BIN) $(WINDOWS_OUTPUT_GOST_ALIAS)
	cp $(WINDOWS_WEBVIEW2_LOADER) $(WINDOWS_OUTPUT_WEBVIEW2_LOADER)
	@echo "Windows executable built at $(WINDOWS_OUTPUT_BIN)"
	@echo "Windows gost sidecar copied to $(WINDOWS_OUTPUT_GOST)"
	@echo "Windows gost alias copied to $(WINDOWS_OUTPUT_GOST_ALIAS)"
	@echo "WebView2 loader copied to $(WINDOWS_OUTPUT_WEBVIEW2_LOADER)"

clean:
	rm -rf $(DIST_DIR) $(TAURI_DIR)/target $(BIN_DIR)
