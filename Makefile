BINARY_NAME=qc

.DEFAULT_GOAL := build

build:
	cargo build --release

install:
	cargo install --path .
	@echo "✅ Installed $(BINARY_NAME) via cargo"

uninstall:
	cargo uninstall $(BINARY_NAME)
	@echo "🗑️  Uninstalled $(BINARY_NAME)"

copy:
	@command -v wl-copy >/dev/null || (echo "wl-copy not found"; exit 1)
	@find . -name "*.rs" -not -path "./target/*" -exec printf "\n### FILE: %s\n\`\`\`rust\n" {} \; -exec cat {} \; -exec printf "\n\`\`\`\n" \; | wl-copy
	@echo "✅ All .rs files bundled and copied."

clean:
	cargo clean

.PHONY: build install uninstall copy clean
