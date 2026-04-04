# Repository Guidelines

## Project Structure & Module Organization
`slick` is a single-crate Rust CLI for an async Zsh prompt. Core code lives in `src/`: `src/bin/slick.rs` exposes the `precmd` and `prompt` subcommands, `src/precmd.rs` collects git state, `src/prompt.rs` renders the prompt, and `src/git.rs` holds git-specific helpers. Integration and regression tests live in `tests/*.rs`, while parser/helper unit tests sit next to the code under `#[cfg(test)]`. Benchmarks are in `benches/`, shell integration helpers in `slick.zsh`, `slick.plugin.zsh`, `load.zsh`, and `test.sh`, and macOS assets/docs in `macOs/`.

## Build, Test, and Development Commands
- `cargo build --release`: build the optimized `slick` binary.
- `cargo test`: run Rust unit and integration tests.
- `cargo clippy --all-targets --all-features`: run the strict lint gate used by the project.
- `cargo fmt --all` or `cargo fmt --all --check`: format or verify formatting.
- `just test`: full local gate; runs clippy, release build, `cargo test`, and `./test.sh`.
- `source slick.zsh`: load the reusable prompt integration from this checkout.
- `source load.zsh`: load the locally built prompt into your current shell for repo-local manual testing.

## Coding Style & Naming Conventions
Use Rust 2024 with default `rustfmt` formatting (4-space indentation). Follow existing `snake_case` naming for functions, variables, and test names. Prefer small helpers over deeply nested prompt logic. Clippy is intentionally strict: warnings are denied, and production code should avoid `unwrap`, `expect`, `panic`, and unchecked indexing. Keep environment-variable names in the existing `SLICK_PROMPT_*` style, and preserve compatibility behavior explicitly when changing prompt parsing or rendering.

## Testing Guidelines
Add unit tests for parser/helper behavior and end-to-end tests for visible prompt output. Name tests `test_<behavior>`, for example `test_pyenv_system_only_marker_is_suppressed`. When changing prompt rendering, cover both the low-level parser and the rendered prompt string in `tests/toolbox_prompt_tests.rs`. Run `cargo test` for fast iteration and `just test` before sending changes.

## Commit & Pull Request Guidelines
Recent history uses short, direct commit subjects such as `cargo fmt`, `fix #19 cargo clippy & fmt`, and version bumps like `0.17.0`. Keep commits focused on one concern. Pull requests should target the `develop` branch; the repo follows git-flow. In the PR description, summarize user-visible prompt changes, list new or changed environment variables, link related issues, and include a prompt sample or screenshot when rendering changes are involved.

## Configuration Notes
Use `envrc` and `README.md` as the source of truth for prompt configuration examples. Prefer `SLICK_PROMPT_PYTHON_ENV_COLOR` over the legacy `PIPENV_ACTIVE_COLOR` fallback when documenting Python prompt settings.

## Agent Workflow
Before finishing any code change, always run:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features`

If formatting is not clean, run `cargo fmt --all` and rerun the checks. Treat these commands as the minimum validation gate even for small prompt, parser, or test updates.
