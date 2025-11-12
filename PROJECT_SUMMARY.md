# llama.cpp-rust - Project Summary

## Overview

A production-ready Rust wrapper for llama.cpp that follows the same pattern as catboost-rust, lightgbm-rust, and xgboost-rust. This crate provides fast builds through pre-built binary downloads instead of compiling from source.

## Key Features

✅ **Dynamic Linking with Binary Downloads**
- Downloads pre-built binaries from llama.cpp releases
- Extracts all necessary libraries (libllama + libggml dependencies)
- ~50s build time vs 5+ minutes for compile-from-source

✅ **Multi-Version Support**
- Supports any llama.cpp version (default: b7035)
- Version-aware header downloading
- Conditional compilation based on API changes
- Tested with b7035, b7000, b6739, b6000+

✅ **Thread Safety**
- Properly documented as NOT thread-safe (llama.cpp limitation)
- Users must use Arc<Mutex<>> or separate instances per thread

✅ **Comprehensive Testing**
- Unit tests for API functionality
- Doc tests for examples
- CI testing across multiple platforms and versions

✅ **CI/CD Pipeline**
- GitHub Actions workflow
- Tests on Linux, macOS, Windows
- Multiple llama.cpp version testing
- Documentation building
- Code formatting and linting

## Project Structure

```
llama.cpp-rust/
├── .github/workflows/ci.yml    # CI configuration
├── src/
│   ├── lib.rs                  # Public API + tests
│   ├── sys.rs                  # FFI bindings (generated)
│   ├── error.rs                # Error handling
│   └── model.rs                # Safe Rust wrappers
├── examples/
│   └── simple.rs               # Example usage
├── build.rs                    # Build script (430+ lines)
├── wrapper.h                   # C header wrapper
├── Cargo.toml                  # Package manifest
├── README.md                   # User documentation
├── VERSIONS.md                 # Version support guide
├── CONTRIBUTING.md             # Contributor guide
├── LICENSE                     # MIT license
└── .gitignore                  # Git ignore patterns
```

## Build Process

### 1. Version Detection
- Reads `LLAMA_CPP_VERSION` env var (default: b7035)
- Parses version number for conditional compilation
- Emits cargo cfg flags

### 2. Platform Detection
- Determines OS (macos/ubuntu/windows)
- Determines architecture (x86_64/aarch64)

### 3. Header Download
- Downloads version-appropriate headers from GitHub
- Handles version-specific headers gracefully
- Caches downloads to speed up rebuilds

### 4. Binary Download & Extraction
- Downloads release zip from GitHub
- Extracts ALL library files:
  - libllama (main library)
  - libggml and variants (dependencies)
  - libggml-metal, libggml-cpu, etc. (backends)
- Sets execute permissions on Unix

### 5. Binding Generation
- Uses bindgen to generate FFI bindings
- Filters to llama_* and ggml_* functions/types
- Outputs to OUT_DIR/bindings.rs

### 6. Library Copying
- Copies all libraries to target/{debug|release}
- Ensures libraries are available at runtime

### 7. RPATH Configuration
- **macOS**: @executable_path, @loader_path rpaths
- **Linux**: $ORIGIN rpaths
- **Windows**: DLL search path
- Modifies install_name on macOS for all dylibs

## API Design

### Safe Wrappers
- `LlamaModel` - RAII wrapper for model handle
- `LlamaContext` - RAII wrapper for context handle
- `ModelParams` - Builder pattern for model loading
- `ContextParams` - Builder pattern for context creation

### Error Handling
- `LlamaError` enum with detailed error types
- `LlamaResult<T>` type alias
- Implements std::error::Error trait

### Version Compatibility
- Uses `llama_model_get_vocab()` for b6739+ compatibility
- Conditional compilation for version-specific APIs
- Graceful degradation for older versions

## Testing Strategy

### Unit Tests
- Backend initialization/cleanup
- Parameter builders
- Error handling
- No model required

### Doc Tests
- Example code in documentation
- Ensures examples stay up-to-date

### CI Testing
- Multiple platforms (Linux, macOS, Windows)
- Multiple llama.cpp versions (b7035, b7000, b6739)
- Documentation building
- Code quality checks

## Comparison with Other Crates

| Feature | llama-cpp-rust | llama-cpp-2 | llama-cpp-sys |
|---------|----------------|-------------|---------------|
| Build Time | ~50s | ~5min | ~5min |
| C++ Compiler | ❌ Not needed | ✅ Required | ✅ Required |
| Binary Size | Small | Large | Large |
| Approach | Download binaries | Compile source | Compile source |
| Version Switching | Instant | Slow rebuild | Slow rebuild |

## Known Limitations

1. **Thread Safety**: llama.cpp C API is not thread-safe
   - Removed Send/Sync implementations
   - Users must handle synchronization

2. **Model Loading**: Requires GGUF model files
   - Cannot test without real models
   - Integration tests need test fixtures

3. **Version Coverage**: Focused on b5000+
   - Older versions may not have pre-built binaries
   - API changes before b5000 not handled

## Future Enhancements

Potential improvements:

1. **More API Coverage**
   - Sampling strategies (top-k, top-p, temperature)
   - Batch processing utilities
   - Model quantization
   - Fine-tuning support

2. **Better Testing**
   - Integration tests with tiny test models
   - Benchmark suite
   - Memory leak detection

3. **Documentation**
   - More examples (chat, completion, embedding)
   - Performance tuning guide
   - Troubleshooting guide

4. **Tooling**
   - Model download utilities
   - Format conversion helpers
   - Performance profiling

## Credits

- Inspired by catboost-rust, lightgbm-rust, xgboost-rust
- Built on [llama.cpp](https://github.com/ggml-org/llama.cpp) by Georgi Gerganov
- Uses [bindgen](https://github.com/rust-lang/rust-bindgen) for FFI generation

## License

MIT License - see LICENSE file

## Contributing

See CONTRIBUTING.md for development guidelines.
