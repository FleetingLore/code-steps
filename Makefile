.PHONY: publish check build

# Publish both crates in the correct order (macros first).
publish:
	@echo "=== Publishing code-steps-macros ==="
	cargo publish -p code-steps-macros
	@echo "=== Publishing code-steps ==="
	cargo publish -p code-steps

# Dry-run before publishing.
check:
	cargo publish --dry-run -p code-steps-macros
	cargo publish --dry-run -p code-steps

# Build all examples.
build:
	cargo build --examples
