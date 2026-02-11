BINARY_NAME=qc
INSTALL_PATH=$(HOME)/.cargo/bin

.DEFAULT_GOAL := release

release:
	cargo build --release

install:
	cargo install --path .	
	@echo "✅ Installed $(BINARY_NAME) to $(INSTALL_PATH)"

uninstall:
	@rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "🗑️  Uninstalled $(BINARY_NAME)"

.PHONY: release install uninstall
