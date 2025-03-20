test: clippy
  cargo test

clippy:
  cargo clippy --all -- -W clippy::all -W clippy::nursery -D warnings
