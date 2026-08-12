.PHONY: cargo-target-dir install install-local build test lint fmt check run release release-tag release-push release-publish vibe-kernel-set vibe-kernel-path vibe-pull vibe pull

PROJECT_NAME := $(notdir $(CURDIR))
BIN_NAME := $(PROJECT_NAME)
CONSTRUCTION_SIDE := $(HOME)/construction_side
CARGO_TARGET_DIR := $(CONSTRUCTION_SIDE)/$(PROJECT_NAME)/target
INSTALL_HOME_KIND ?= x-cli
INSTALL_DIR ?= $(HOME)/.$(INSTALL_HOME_KIND)-$(PROJECT_NAME)
export CARGO_TARGET_DIR

cargo-target-dir:
	@mkdir -p "$(CARGO_TARGET_DIR)"

install: install-local

install-local: build
	@mkdir -p "$(INSTALL_DIR)"
	@tmp="$(INSTALL_DIR)/.$(BIN_NAME).tmp.$$$$"; \
	trap 'rm -f "$$tmp"' EXIT HUP INT TERM; \
	cp "$(CARGO_TARGET_DIR)/release/$(BIN_NAME)" "$$tmp"; \
	chmod 0755 "$$tmp"; \
	mv -f "$$tmp" "$(INSTALL_DIR)/$(BIN_NAME)"; \
	trap - EXIT HUP INT TERM; \
	printf "Installed %s\n" "$(INSTALL_DIR)/$(BIN_NAME)"

build: cargo-target-dir
	cargo build --release

test: cargo-target-dir
	cargo test

lint: cargo-target-dir
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

check: fmt lint test

run: cargo-target-dir
	cargo run

release:
	@echo "TODO: implement plain release with GitHub Release CI/CD."; exit 1

release-tag:
	@echo "TODO: implement release tagging for GitHub Release CI/CD."; exit 1

release-push:
	@echo "TODO: implement release push that triggers GitHub Release CI/CD."; exit 1

release-publish:
	@echo "TODO: implement CI/CD release artifact publish handoff."; exit 1

vibe-kernel-path:
	@if [ ! -f ".vibe/KERNEL_SOURCE" ]; then \
		echo "Missing .vibe/KERNEL_SOURCE."; \
		echo "Run: make vibe-kernel-set"; \
		exit 1; \
	fi; \
	printf "%s\n" "$$(cat .vibe/KERNEL_SOURCE)"

vibe-kernel-set:
	@mkdir -p .vibe; \
	if [ -n "$(KERNEL)" ]; then kernel_root="$(KERNEL)"; else \
		printf "Kernel path (absolute, contains tools/vibe-pull): " ; \
		read -r kernel_root; \
	fi; \
	if [ -z "$$kernel_root" ]; then echo "ERROR: empty path." >&2; exit 1; fi; \
	case "$$kernel_root" in /*) ;; *) echo "ERROR: must be an absolute path." >&2; exit 1;; esac; \
	if [ ! -f "$$kernel_root/tools/vibe-pull" ]; then echo "ERROR: cannot find $$kernel_root/tools/vibe-pull" >&2; exit 1; fi; \
	printf "%s\n" "$$kernel_root" > .vibe/KERNEL_SOURCE; \
	echo "Wrote .vibe/KERNEL_SOURCE"

vibe-pull:
	@if [ ! -f ".vibe/KERNEL_SOURCE" ]; then \
		echo "Missing .vibe/KERNEL_SOURCE."; \
		echo "Run: make vibe-kernel-set"; \
		exit 1; \
	fi; \
	kernel_root="$$(cat .vibe/KERNEL_SOURCE)"; \
	if [ ! -f "$$kernel_root/tools/vibe-pull" ]; then \
		echo "ERROR: cannot find $$kernel_root/tools/vibe-pull"; \
		exit 1; \
	fi; \
	python3 "$$kernel_root/tools/vibe-pull" .

vibe:
	@case " $(MAKECMDGOALS) " in *" pull "*) \
		:; \
		;; \
	*) \
		echo "Usage: make vibe pull"; \
		exit 2; \
		;; \
	esac

pull:
	@case " $(MAKECMDGOALS) " in *" vibe "*) \
		$(MAKE) vibe-pull; \
		;; \
	*) \
		echo "Usage: make vibe pull"; \
		exit 2; \
		;; \
	esac
