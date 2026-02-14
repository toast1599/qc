BINARY_NAME=qc
# Use ?= so it can be overridden from the CLI if needed
INSTALL_PATH ?= $(HOME)/.cargo/bin

.DEFAULT_GOAL := release

release:
	cargo build --release

# 'cargo install' is safer than manual copying as it handles binary stripping
install:
	cargo install --path .	
	@echo "✅ Installed $(BINARY_NAME) to $(INSTALL_PATH)"

uninstall:
	@rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "🗑️  Uninstalled $(BINARY_NAME)"

copy:
	@echo "#--WAYLAND-ONLY--#"
	@find . -name "*.rs" -not -path "./target/*" -exec sh -c 'echo "--- FILE: {} ---"; cat {}; echo "\n"' \; | wl-copy
	@echo "✅ All .rs files bundled and copied to wl-clipboard."

clean:
	cargo clean

.PHONY: release install uninstall copy clean
