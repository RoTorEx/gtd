.PHONY: cargo-target-dir install install-local setup-path build test lint fmt check run vibe-kernel-set vibe-kernel-path vibe-pull vibe pull

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
	@$(MAKE) setup-path

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
		echo '  export PATH="$(INSTALL_DIR):$$PATH"'; \
		exit 0; \
	fi; \
	block_start="# x-cli-$(PROJECT_NAME)"; \
	block_end="# /x-cli-$(PROJECT_NAME)"; \
	if grep -qF "$$block_start" "$$profile" 2>/dev/null; then \
		echo "PATH block already present in $$profile"; \
	else \
		printf '\n%s\nexport PATH="$(INSTALL_DIR):$$PATH"\n%s\n' "$$block_start" "$$block_end" >> "$$profile"; \
		echo "Added $(INSTALL_DIR) to PATH in $$profile"; \
	fi; \
	echo 'Reload your shell or run: export PATH="$(INSTALL_DIR):$$PATH"'

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
