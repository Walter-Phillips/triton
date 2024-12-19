# Code Style and Structure
- Write concise, idiomatic Rust code with accurate examples.
- Prefer functional and iterator-based approaches over imperative loops.
- Use `match` for control flow and pattern matching instead of nested `if` statements.
- Favor modular code with a clear separation of concerns:
  - Place the main application logic in `src/main.rs`.
  - Abstract functionality into modules in `src/lib.rs` or dedicated submodules.
  - Organize tests in `tests/` directory and use `#[cfg(test)]` for unit tests within modules.
- Use descriptive variable names in snake_case.
- Favor explicit over implicit behavior; avoid macros for non-obvious abstractions.

# Naming Conventions
- Use snake_case for functions, variables, and file names (e.g., `user_service.rs`).
- Use PascalCase for structs, enums, and traits.
- Use UPPER_SNAKE_CASE for constants and statics.
- Prefix private module names with an underscore only when necessary.

# Error Handling
- Prioritize proper error handling:
  - Use `Result<T, E>` and `Option<T>` for recoverable and optional values.
  - Model errors with `thiserror` or `anyhow` for detailed contexts.
  - Avoid panicking in library code; use `unwrap` and `expect` only in examples or tests.
  - Implement `From` and `Into` for custom error types where appropriate.
- Use `eyre` for ergonomic error handling in applications, where applicable.

# Syntax and Formatting
- Use `rustfmt` for consistent formatting; enforce it with CI.
- Always include type annotations where inference might not be clear.
- Use `impl Trait` for return types to simplify signatures.
- Prefer `for` loops with iterators over `while` or manual indexing.
- Avoid unnecessary clones; use references or smart pointers (e.g., `Rc`, `Arc`) judiciously.

# Testing and Documentation
- Write tests for all critical logic:
  - Use `assert_eq!`, `assert_ne!`, and `matches!` for testing conditions.
  - Use property-based testing with `proptest` for edge cases.
- Document public APIs with `///` doc comments:
  - Include examples and usage patterns in the documentation.
  - Use `cargo doc` to verify generated documentation.
- Mark modules and functions with `#[doc(hidden)]` if they should not appear in public docs.

# Libraries and Ecosystem
- Use libraries recommended by blessed.rs:
  - For async programming: `tokio` or `async-std` for runtime, `reqwest` for HTTP.
  - For error handling: `thiserror`, `anyhow`, or `eyre`.
  - For CLI tools: `clap` or `structopt`.
  - For serialization: `serde` and `serde_json`.
  - For testing: `proptest` and `assert_cmd`.
  - For database access: `sqlx` or `diesel`.
- Follow Rust edition best practices (`2021` edition recommended).

# Performance Optimization
- Avoid premature optimization; profile with `cargo flamegraph` or `perf`.
- Use `Arc` and `Mutex` for shared state only when necessary; prefer immutable data.
- Avoid dynamic dispatch unless flexibility is required; prefer static dispatch and generics.
- Use `lazy_static` or `once_cell` for expensive initializations.

# Key Conventions
- Follow Rust’s ownership and borrowing principles:
  - Minimize `clone` usage; prefer borrowing where possible.
  - Use `Cow` for zero-cost abstraction when working with owned/borrowed data.
- Use `#[derive]` macros for common traits like `Debug`, `Clone`, `PartialEq`, and `Serialize`.
- Optimize binaries with `cargo build --release` for production.
- Favor iterators over manual loops:
  - Use `.map()`, `.filter()`, and `.fold()` for transforming collections.
  - Avoid `collect` unless necessary; prefer lazy evaluations with `Iterator`.

# UI and Frameworks (if applicable)
- For GUI applications, use `egui` or `iced`.
- For web applications, use `yew` or `leptos` for a declarative frontend approach.

# Documentation and Deployment
- Document features in `README.md` with usage examples.
- Generate changelogs with `git-cliff` or `cargo-release`.
- Use CI for building, testing, linting, and formatting (`GitHub Actions` recommended).
- Publish libraries to `crates.io` with well-maintained `Cargo.toml`.

# Miscellaneous
- Use `cargo clippy` to enforce linting standards.
- Use `cargo fmt` and `cargo fix` to maintain clean code.
- Leverage `cargo audit` for dependency security checks.
