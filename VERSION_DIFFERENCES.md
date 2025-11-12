# llama.cpp Version Differences

This document details the key API changes between different llama.cpp versions and how llama-cpp-rust handles them.

## Quick Reference

| Version | Release Date | Key Changes | Support Status |
|---------|-------------|-------------|----------------|
| b7035 | Nov 2025 | Latest stable | ✅ Fully Tested |
| b7000 | Oct 2025 | Performance improvements | ✅ Tested |
| b6739 | Sep 2025 | **Vocab API separated** | ✅ Tested (Breaking) |
| b6000 | Aug 2025 | New optimization headers | ✅ Supported |
| b5000 | Jun 2025 | Modern header structure | ⚠️ Basic Support |

## Major Breaking Changes

### b6739: Vocabulary API Separation

**What Changed:**
The vocabulary-related functions were separated from the model API. This is the most significant breaking change.

**Before (b6738 and earlier):**
```c
// Functions took llama_model* directly
int32_t llama_n_vocab(const struct llama_model * model);
int32_t llama_tokenize(
    const struct llama_model * model,
    const char * text,
    ...
);
```

**After (b6739+):**
```c
// New vocab type introduced
const struct llama_vocab * llama_model_get_vocab(const struct llama_model * model);

// Functions now take llama_vocab*
int32_t llama_n_vocab(const struct llama_vocab * vocab);
int32_t llama_tokenize(
    const struct llama_vocab * vocab,
    const char * text,
    ...
);
```

**How llama-cpp-rust Handles It:**
The Rust wrapper automatically handles this internally:

```rust
// User-facing API stays the same
pub fn n_vocab(&self) -> i32 {
    let vocab = unsafe { sys::llama_model_get_vocab(self.model) };
    unsafe { sys::llama_n_vocab(vocab) }
}
```

**Conditional Compilation:**
```rust
#[cfg(llama_vocab_api)]  // Defined for b6739+
fn use_new_api() { ... }

#[cfg(not(llama_vocab_api))]  // For older versions
fn use_old_api() { ... }
```

## Header Changes

### Evolution of Required Headers

**All Versions:**
- `llama.h` - Core API
- `ggml.h` - GGML tensor library

**b5000+:**
- `ggml-cpu.h` - CPU backend
- `ggml-backend.h` - Backend abstraction
- `ggml-alloc.h` - Memory allocation

**b6000+:**
- `ggml-opt.h` - Optimization routines

**How It's Handled:**
The build script only downloads headers that exist for the target version:

```rust
// Version-specific headers
if version_num >= 5000 {
    headers.push(("ggml/include/ggml-cpu.h", "ggml-cpu.h", 5000));
    headers.push(("ggml/include/ggml-backend.h", "ggml-backend.h", 5000));
    headers.push(("ggml/include/ggml-alloc.h", "ggml-alloc.h", 5000));
}

if version_num >= 6000 {
    headers.push(("ggml/include/ggml-opt.h", "ggml-opt.h", 6000));
}
```

## Library Structure Changes

### Pre-built Binary Contents

**b7035 (Current):**
```
llama-b7035-bin-macos-arm64.zip contains:
├── build/bin/
│   ├── libllama.dylib              # Main library
│   ├── libllama.0.dylib            # Versioned symlink
│   ├── libllama.0.0.7035.dylib     # Full version
│   ├── libggml.dylib               # GGML base
│   ├── libggml.0.dylib
│   ├── libggml.0.9.4.dylib
│   ├── libggml-cpu.dylib           # CPU backend
│   ├── libggml-cpu.0.dylib
│   ├── libggml-metal.dylib         # Metal backend (macOS)
│   ├── libggml-blas.dylib          # BLAS backend
│   ├── libggml-base.dylib          # Base utilities
│   └── libggml-rpc.dylib           # RPC support
```

**Earlier Versions (b6000-b6999):**
- Fewer backend libraries
- No RPC support
- Simpler structure

## Function Signature Changes

### Context Parameters

**b6000+:**
```c
struct llama_context_params {
    uint32_t n_ctx;           // Changed from int32_t
    uint32_t n_batch;
    int32_t  n_threads;       // Still int32_t
    int32_t  n_threads_batch;
    // ... more fields
};
```

**How llama-cpp-rust Handles It:**
Type conversions in the builder:

```rust
pub fn n_ctx(mut self, n: u32) -> Self {
    self.inner.n_ctx = n;  // Direct assignment (matching type)
    self
}

pub fn n_threads(mut self, n: u32) -> Self {
    self.inner.n_threads = n as i32;  // Type conversion
    self
}
```

## Performance Improvements by Version

### b7000+
- Improved Metal backend performance
- Better memory efficiency
- Faster prompt processing

### b6739+
- Vocab API allows better caching
- Reduced memory overhead for multi-context scenarios

### b6000+
- New optimization routines
- Better quantization support

## Testing Different Versions

### Building with Specific Versions

```bash
# Latest (default)
cargo build

# Specific version
LLAMA_CPP_VERSION=b7000 cargo build

# Older version
LLAMA_CPP_VERSION=b6739 cargo build
```

### Version-Specific Tests

```bash
# Test multiple versions
for version in b7035 b7000 b6739; do
    echo "Testing $version..."
    LLAMA_CPP_VERSION=$version cargo test
done
```

## Compatibility Matrix

| Your Code | b7035 | b7000 | b6739 | b6000 | b5000 |
|-----------|-------|-------|-------|-------|-------|
| Basic model loading | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tokenization | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Inference | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Advanced features | ✅ | ✅ | ⚠️ | ⚠️ | ❌ |

Legend:
- ✅ Fully supported
- ⚠️ May have limitations
- ❌ Not supported

## Migration Guide

### Upgrading from b6738 to b6739+

**No changes needed!** The Rust wrapper handles the vocab API change automatically.

**If you were using raw FFI:**
```rust
// Old way (b6738-)
let n = unsafe { sys::llama_n_vocab(model) };

// New way (b6739+)
let vocab = unsafe { sys::llama_model_get_vocab(model) };
let n = unsafe { sys::llama_n_vocab(vocab) };
```

**Using the safe wrapper (works for all versions):**
```rust
let n = model.n_vocab();  // Always works!
```

### Downgrading Versions

If you need to use an older version temporarily:

```bash
# Switch to older version
LLAMA_CPP_VERSION=b6739 cargo build

# Your code continues to work
cargo run --example simple
```

## Common Issues

### "Header not found"

**Symptom:**
```
error: ggml-opt.h: No such file or directory
```

**Cause:** Using a version older than b6000

**Solution:** Use b6000+ or let the build script skip optional headers

### "Library not found in archive"

**Symptom:**
```
error: Main library libllama.dylib not found in archive
```

**Cause:** Version doesn't have pre-built binaries for your platform

**Solution:** Use b6000+ which has consistent binary releases

### API Mismatch Errors

**Symptom:**
```
error: cannot find function `llama_tokenize` in module `sys`
```

**Cause:** Build cache from different version

**Solution:**
```bash
cargo clean
LLAMA_CPP_VERSION=b7035 cargo build
```

## Future-Proofing

### How to Prepare for Future Versions

1. **Use the safe wrapper API** - It handles version differences
2. **Avoid raw FFI** unless necessary
3. **Pin versions in production:**
   ```bash
   export LLAMA_CPP_VERSION=b7035
   ```
4. **Test before upgrading:**
   ```bash
   LLAMA_CPP_VERSION=b7100 cargo test
   ```

### Reporting Version Issues

If you encounter issues with a new version:

1. Check if pre-built binaries exist
2. Verify header availability
3. Open an issue with:
   - Version number
   - Error message
   - Platform details

## Version-Specific Features

### Conditional Compilation Example

Currently, the only version-specific cfg flag is `llama_vocab_api` for the b6739 vocab API change:

```rust
#[cfg(llama_vocab_api)]
pub fn efficient_tokenization() {
    // Available in b6739+
    // Uses the new vocab API
    let vocab = unsafe { llama_model_get_vocab(model) };
    llama_tokenize(vocab, text, ...)
}

#[cfg(not(llama_vocab_api))]
pub fn efficient_tokenization() {
    // Fallback for older versions (before b6739)
    // Uses the old API that takes model directly
    llama_tokenize(model, text, ...)
}
```

**Note:** As new breaking changes are introduced in llama.cpp, we'll add more cfg flags as needed.

## Summary

**Key Takeaways:**
- ✅ Wrapper handles version differences automatically
- ✅ b6739 vocab API change is the main breaking change
- ✅ b5000+ recommended, b6739+ ideal
- ✅ Headers are version-aware
- ✅ Use environment variable to switch versions
- ✅ Safe API works across all supported versions

**Best Practice:**
Use the latest tested version (b7035) for new projects, pin it in production, and test before upgrading.
