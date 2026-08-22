.PHONY: cargo-target-dir install install-local setup-config setup-path build test lint fmt check run version release release-tag release-push vibe-kernel-set vibe-kernel-path vibe-pull vibe pull

PROJECT_NAME := $(notdir $(CURDIR))
BIN_NAME := $(PROJECT_NAME)
X_CLI_HOME := $(HOME)/.x-cli-$(PROJECT_NAME)
X_CLI_BIN_DIR := $(X_CLI_HOME)/bin
GTD_CONFIG_DIR := $(if $(XDG_CONFIG_HOME),$(XDG_CONFIG_HOME),$(HOME)/.config)/gtd
GTD_CONFIG_FILE := $(GTD_CONFIG_DIR)/config.toml
CONSTRUCTION_SIDE := $(HOME)/construction_side
CARGO_TARGET_DIR := $(CONSTRUCTION_SIDE)/$(PROJECT_NAME)/target
export CARGO_TARGET_DIR

cargo-target-dir:
	@mkdir -p "$(CARGO_TARGET_DIR)"

install: install-local

install-local: build setup-config
	@mkdir -p "$(X_CLI_BIN_DIR)"
	@tmp="$(X_CLI_BIN_DIR)/.$(BIN_NAME).tmp.$$$$"; \
	trap 'rm -f "$$tmp"' EXIT HUP INT TERM; \
	cp "$(CARGO_TARGET_DIR)/release/$(BIN_NAME)" "$$tmp"; \
	chmod 0755 "$$tmp"; \
	mv -f "$$tmp" "$(X_CLI_BIN_DIR)/$(BIN_NAME)"; \
	trap - EXIT HUP INT TERM; \
	printf "Installed %s\n" "$(X_CLI_BIN_DIR)/$(BIN_NAME)"
	@$(MAKE) setup-path

setup-config:
	@mkdir -p "$(GTD_CONFIG_DIR)"
	@if [ ! -f "$(GTD_CONFIG_FILE)" ]; then \
		cp config.example.toml "$(GTD_CONFIG_FILE)"; \
		printf "Created %s with the classic theme\n" "$(GTD_CONFIG_FILE)"; \
	else \
		printf "Preserved existing %s\n" "$(GTD_CONFIG_FILE)"; \
	fi

setup-path:
	@profile=""; \
	if [ -f "$(HOME)/.zshrc" ]; then profile="$(HOME)/.zshrc"; \
	elif [ -f "$(HOME)/.bashrc" ]; then profile="$(HOME)/.bashrc"; \
	else \
		case "$$(basename "$$SHELL")" in \
			zsh) profile="$(HOME)/.zshrc" ;; \
			bash) profile="$(HOME)/.bashrc" ;; \
		esac; \
	fi; \
	if [ -z "$$profile" ]; then \
		echo "Could not detect shell profile. Add this manually:"; \
		echo '  export PATH="$(X_CLI_BIN_DIR):$$PATH"'; \
		exit 0; \
	fi; \
	block_start="# x-cli-$(PROJECT_NAME)"; \
	block_end="# /x-cli-$(PROJECT_NAME)"; \
	if grep -qF "$$block_start" "$$profile" 2>/dev/null; then \
		awk -v start="$$block_start" -v end="$$block_end" '$$0 == start { skip = 1; next } $$0 == end { skip = 0; next } !skip' "$$profile" > "$$profile.gtd.tmp" && mv "$$profile.gtd.tmp" "$$profile"; \
	fi; \
	printf '\n%s\nexport PATH="$(X_CLI_BIN_DIR):$$PATH"\n%s\n' "$$block_start" "$$block_end" >> "$$profile"; \
	echo "Updated $(X_CLI_BIN_DIR) PATH block in $$profile"; \
	echo 'Reload your shell or run: export PATH="$(X_CLI_BIN_DIR):$$PATH"'

build: cargo-target-dir
	cargo build --release

test: cargo-target-dir
	cargo test

lint: cargo-target-dir
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

check: cargo-target-dir
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test

run: cargo-target-dir
	cargo run

version: cargo-target-dir
	cargo run --locked -- --version

release:
	sh scripts/release.sh

release-tag:
	@set -eu; \
	branch="$$(git branch --show-current)"; \
	test "$$branch" = "main" || { echo "ERROR: release tag must be created from main, not $$branch" >&2; exit 1; }; \
	test -z "$$(git status --porcelain)" || { echo "ERROR: commit or remove local changes before tagging" >&2; exit 1; }; \
	version="$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"; \
	test -n "$$version" || { echo "ERROR: could not read version from Cargo.toml" >&2; exit 1; }; \
	! git rev-parse --verify "refs/tags/v$$version" >/dev/null 2>&1 || { echo "ERROR: tag v$$version already exists" >&2; exit 1; }; \
	git tag -a "v$$version" -m "Release $$version"; \
	echo "Created annotated tag v$$version"

release-push:
	@set -eu; \
	branch="$$(git branch --show-current)"; \
	test "$$branch" = "main" || { echo "ERROR: releases must be pushed from main, not $$branch." >&2; exit 1; }; \
	version="$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"; \
	tag="v$$version"; \
	git rev-parse -q --verify "refs/tags/$$tag" >/dev/null || { echo "ERROR: missing $$tag. Run make release." >&2; exit 1; }; \
	git push origin main --follow-tags

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

# VIBE:KERNEL_MAKE_START

.PHONY: vibe-propose

vibe-propose:
	@test -f .vibe/KERNEL_SOURCE || { echo "Missing .vibe/KERNEL_SOURCE. Run: make vibe-kernel-set" >&2; exit 1; }
	@kernel_root="$$(sed -n '1p' .vibe/KERNEL_SOURCE)"; \
	test -f "$$kernel_root/tools/vibe-propose" || { echo "Missing $$kernel_root/tools/vibe-propose. Update the kernel source first." >&2; exit 1; }; \
	python3 "$$kernel_root/tools/vibe-propose" .

# VIBE:KERNEL_MAKE_END
