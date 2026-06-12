# Publish both crates in the correct order (macros first, then code-steps).
# Usage: just publish
publish:
	@echo "=== Publishing code-steps-macros ==="
	cargo publish -p code-steps-macros
	@echo "=== Publishing code-steps ==="
	cargo publish -p code-steps

# Dry-run both crates to verify before publishing.
check:
	cargo publish --dry-run -p code-steps-macros
	cargo publish --dry-run -p code-steps

# Build everything.
build:
	cargo build --examples

# Run the nested steps demo.
demo:
	cargo run --example nested

# Bump version. Usage: just bump 0.4.2
bump V:
	sed -i '' 's/^version = ".*"/version = "{{V}}"/' Cargo.toml
	@echo "Version bumped to {{V}}. Don't forget to update code-steps/Cargo.toml dependency."

default: build
