# Version Support

This document describes how `llama-cpp-rust` supports multiple versions of llama.cpp.

## How Version Support Works

Unlike most libraries that use semantic versioning (e.g., v1.2.3), llama.cpp uses **build tags** like `b7035`, `b6739`, etc. These represent sequential build numbers.

### Default Version

The default version is **b7035** (released Nov 12, 2025).

### Using Different Versions

Set the `LLAMA_CPP_VERSION` environment variable during build:

```bash
# Use a specific version
LLAMA_CPP_VERSION=b7000 cargo build

# Use the latest release
LLAMA_CPP_VERSION=b7100 cargo build
```

## Version Detection

The build script automatically:

1. **Parses the version** - Extracts numeric build number (e.g., `b7035` → `7035`)
2. **Downloads appropriate headers** - Skips headers not available in older versions
3. **Emits cfg flags** - For conditional compilation in your code

### Available cfg Flags

| Flag | Version | Description |
|------|---------|-------------|
| `llama_vocab_api` | b6739+ | Separate vocab API (breaking change) |

**Note:** Only flags that are actually used in the code are emitted. As new breaking changes occur in llama.cpp, we'll add more flags as needed.

### Using cfg Flags in Your Code

```rust
#[cfg(llama_vocab_api)]
fn tokenize_new_api() {
    // Use llama_model_get_vocab API
}

#[cfg(not(llama_vocab_api))]
fn tokenize_old_api() {
    // Use older tokenization API
}
```

## Supported Version Ranges

| Version Range | Support Level | Notes |
|---------------|---------------|-------|
| b7000+ | ✅ Fully Tested | Current stable releases |
| b6000-b6999 | ✅ Supported | Includes vocab API changes |
| b5000-b5999 | ⚠️ Basic Support | Older headers, may require adjustments |
| b4000-b4999 | ❌ Not Supported | No pre-built binaries available |

## Version-Specific Headers

Different llama.cpp versions require different header files:

| Header | Required Version | Description |
|--------|-----------------|-------------|
| `llama.h` | All | Core API |
| `ggml.h` | All | GGML tensor library |
| `ggml-cpu.h` | b5000+ | CPU backend |
| `ggml-backend.h` | b5000+ | Backend abstraction |
| `ggml-alloc.h` | b5000+ | Memory allocation |
| `ggml-opt.h` | b6000+ | Optimization routines |

The build script automatically downloads only the headers available for your target version.

## Binary Availability

Pre-built binaries are available from llama.cpp GitHub releases for:

- **Platforms**: macOS (x64, ARM64), Linux (x64, ARM64), Windows (x64)
- **Versions**: Typically b5000+ (check releases page for availability)

If a version doesn't have pre-built binaries, the build will fail with a clear error message.

## Known Breaking Changes

### b6739 - Vocab API Separation

The vocab-related functions were separated from the model API:

**Before (b6738 and earlier):**
```c
llama_tokenize(model, text, ...);
llama_n_vocab(model);
```

**After (b6739+):**
```c
vocab = llama_model_get_vocab(model);
llama_tokenize(vocab, text, ...);
llama_n_vocab(vocab);
```

Our wrapper handles this automatically - the API stays the same, but the implementation changes based on version.

## Testing Different Versions

To test your code against multiple versions:

```bash
# Test with latest
cargo test

# Test with b7000
LLAMA_CPP_VERSION=b7000 cargo test

# Test with b6000
LLAMA_CPP_VERSION=b6000 cargo test
```

## Finding Available Versions

Visit the llama.cpp releases page:
https://github.com/ggml-org/llama.cpp/releases

Look for releases with pre-built binaries (e.g., `llama-bXXXX-bin-*.zip` files).

## Version Pinning

For production, we recommend pinning to a specific version:

```bash
# In your build scripts
export LLAMA_CPP_VERSION=b7035

# Or in a .env file
echo "LLAMA_CPP_VERSION=b7035" >> .env
```

## Troubleshooting

### "Library not found in archive"

**Cause:** The specified version doesn't have pre-built binaries for your platform.

**Solution:** Try a newer version (b6000+) or check the releases page for available versions.

### "Header not found"

**Cause:** Trying to use a version older than b5000.

**Solution:** Use b5000 or newer, which has the modern header structure.

### API Mismatch Errors

**Cause:** The Rust wrapper hasn't been updated for very new API changes.

**Solution:** Use a tested version (b7035, b7000, b6739, b6000) or open an issue.

## Contributing Version Support

To add support for a new llama.cpp version:

1. Update `emit_version_cfgs()` in `build.rs` for any new API changes
2. Update `download_llama_headers()` if new headers are required
3. Test compilation and basic functionality
4. Update `VERSIONS.md` (this file) with test results
5. Submit a PR!

## Version History

| Date | llama.cpp Version | Changes |
|------|-------------------|---------|
| 2025-11-12 | b7035 | Initial release, vocab API support |
| (future) | b7100+ | TBD |
