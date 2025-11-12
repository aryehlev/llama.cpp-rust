# Contributing to llama.cpp-rust

Thank you for your interest in contributing to llama.cpp-rust!

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/llama.cpp-rust.git
   cd llama.cpp-rust
   ```

2. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

## Project Structure

```
llama.cpp-rust/
├── src/
│   ├── lib.rs      # Public API and tests
│   ├── sys.rs      # FFI bindings (auto-generated)
│   ├── error.rs    # Error types
│   └── model.rs    # Safe Rust wrappers
├── examples/
│   └── simple.rs   # Example usage
├── build.rs        # Build script (downloads binaries, generates bindings)
├── wrapper.h       # C header wrapper
└── .github/
    └── workflows/
        └── ci.yml  # CI configuration
```

## Making Changes

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Follow Rust naming conventions

### Testing

- Add tests for new functionality
- Ensure all tests pass: `cargo test`
- Test on multiple platforms if possible

### Documentation

- Add doc comments for public APIs
- Update README.md for user-facing changes
- Update VERSIONS.md for version-specific changes

## Adding Support for New llama.cpp Versions

1. **Update version detection** in `build.rs`:
   ```rust
   fn emit_version_cfgs(version_num: u32) {
       if version_num >= YOUR_VERSION {
           println!("cargo:rustc-cfg=llama_new_feature");
       }
   }
   ```

2. **Update header download** if new headers are needed:
   ```rust
   fn download_llama_headers(...) {
       if version_num >= YOUR_VERSION {
           headers.push(("path/to/new.h", "new.h", YOUR_VERSION));
       }
   }
   ```

3. **Update API wrappers** for breaking changes:
   ```rust
   #[cfg(llama_new_feature)]
   fn new_api() { ... }

   #[cfg(not(llama_new_feature))]
   fn old_api() { ... }
   ```

4. **Test the version**:
   ```bash
   LLAMA_CPP_VERSION=bYOUR_VERSION cargo test
   ```

5. **Update documentation**:
   - Add version to `VERSIONS.md`
   - Update tested versions in README

## Submitting Changes

1. **Fork the repository**

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Write code
   - Add tests
   - Update documentation

4. **Commit your changes**
   ```bash
   git commit -m "Description of changes"
   ```

5. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request**
   - Describe your changes
   - Reference any related issues
   - Ensure CI passes

## Pull Request Guidelines

- **One feature per PR** - Keep PRs focused
- **Write tests** - Ensure new code is tested
- **Update docs** - Keep documentation in sync
- **Pass CI** - All checks must pass
- **Respond to feedback** - Address review comments

## Reporting Issues

When reporting bugs, please include:

- **llama.cpp version** (e.g., b7035)
- **Platform** (OS, architecture)
- **Rust version** (`rustc --version`)
- **Steps to reproduce**
- **Expected vs actual behavior**
- **Error messages** (full output)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow GitHub's Community Guidelines

## Questions?

- Open an issue for questions
- Check existing issues and PRs
- Read the documentation first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing! 🦀
