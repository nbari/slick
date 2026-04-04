# Run all tests
test: clippy build
    cargo test
    ./test.sh

# Run benchmarks
bench:
    cargo bench

# Run clippy with strict warnings
clippy:
  cargo clippy --all-targets --all-features

# Build release binary
build:
  cargo build --release

# Run all checks (test + format)
check: clippy
  cargo test
  cargo fmt -- --check
  ./test.sh

# Format code
fmt:
  cargo fmt

# Preview prompt contexts with current SLICK_PROMPT_* settings
preview:
  zsh scripts/preview_prompt.zsh

# Continuously refresh the prompt preview
preview-watch:
  zsh scripts/preview_prompt.zsh --watch

# Clean build artifacts
clean:
  cargo clean

# Run integration tests only
integration: build
  ./test.sh

# Show version
version:
  @cargo pkgid | cut -d# -f2
