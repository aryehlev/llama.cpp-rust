# API Version Support Matrix

This document details which API functions are available in different llama.cpp versions and how we guard them.

## Core Functions (All Versions)

These functions are available in all supported versions (b5000+):

| Function | Status | Notes |
|----------|--------|-------|
| `LlamaBackend::init()` | ✅ All versions | Backend initialization |
| `LlamaModel::load()` | ✅ All versions | Load GGUF models |
| `LlamaModel::n_ctx_train()` | ✅ All versions | Get training context size |
| `LlamaModel::n_embd()` | ✅ All versions | Get embedding dimension |
| `LlamaContext::new()` | ✅ All versions | Create inference context |
| `LlamaContext::decode()` | ✅ All versions | Run inference |
| `LlamaContext::n_ctx()` | ✅ All versions | Get context size |

## Version-Specific Functions

### Tokenization (b6739+)

**Issue:** The tokenization API changed significantly in b6739 when vocab was separated.

**Status in llama-cpp-rust:** ✅ **Automatically handled**

The wrapper transparently uses the correct API based on version:

```rust
// Works in ALL versions - wrapper handles the difference
pub fn tokenize(&self, text: &str, ...) -> LlamaResult<Vec<i32>> {
    #[cfg(llama_vocab_api)]
    {
        let vocab = unsafe { sys::llama_model_get_vocab(self.model) };
        unsafe { sys::llama_tokenize(vocab, ...) }
    }

    #[cfg(not(llama_vocab_api))]
    {
        unsafe { sys::llama_tokenize(self.model, ...) }
    }
}
```

### Vocabulary Functions (b6739+)

**Functions affected:**
- `n_vocab()` - Get vocabulary size
- `token_to_piece()` - Convert token to text

**Status:** ✅ **Automatically handled** via conditional compilation

## Future Version-Specific Features

When new llama.cpp versions add features, we should guard them:

### Example: Hypothetical Feature in b8000+

```rust
#[cfg(llama_v8000)]
impl LlamaModel {
    /// New feature only available in b8000+
    pub fn new_feature(&self) -> Result<()> {
        // Implementation
    }
}

// Compile-time error prevention
#[cfg(not(llama_v8000))]
impl LlamaModel {
    // Stub or not available
}
```

## Parameter Version Compatibility

Some parameters may not be supported in older versions:

### ModelParams

| Parameter | Version | Status |
|-----------|---------|--------|
| `n_gpu_layers` | All | ✅ Always available |
| `use_mmap` | All | ✅ Always available |
| `use_mlock` | All | ✅ Always available |

### ContextParams

| Parameter | Version | Status |
|-----------|---------|--------|
| `n_ctx` | All | ✅ Always available |
| `n_batch` | All | ✅ Always available |
| `n_threads` | All | ✅ Always available |
| `n_threads_batch` | All | ✅ Always available |

## Implementation Strategy

### 1. Transparent Handling (Preferred)

For breaking API changes, handle them transparently in the wrapper:

```rust
pub fn some_function(&self) -> Result<T> {
    #[cfg(llama_new_api)]
    {
        // New way
    }

    #[cfg(not(llama_new_api))]
    {
        // Old way
    }
}
```

**Pros:**
- User code works across all versions
- No version-specific code in user applications
- Smooth upgrades

**Cons:**
- More complex wrapper implementation

### 2. Feature-Gated APIs (When Necessary)

For genuinely new features that don't exist in older versions:

```rust
#[cfg(llama_new_feature)]
impl SomeStruct {
    /// Only available in llama.cpp b7000+
    ///
    /// # Version Support
    /// This function requires llama.cpp b7000 or later.
    /// Compile with `LLAMA_CPP_VERSION=b7000` or higher.
    pub fn new_feature_function(&self) -> Result<T> {
        // Implementation
    }
}
```

**Pros:**
- Clear what's available when
- Compile-time checking
- No runtime overhead

**Cons:**
- User code may need version guards
- More complex to use

### 3. Runtime Checks (Avoid)

We generally avoid runtime version checks:

```rust
// ❌ Don't do this
pub fn some_function(&self) -> Result<T> {
    if version >= 7000 {
        // new way
    } else {
        // old way
    }
}
```

**Why avoid:**
- Runtime overhead
- Can't catch errors at compile time
- Harder to optimize

## Current Implementation

As of llama-cpp-rust 0.1.0:

### Automatically Handled
- ✅ Vocab API changes (b6739)
- ✅ Header availability (b5000, b6000)
- ✅ Type changes in structs

### User-Visible
- ❌ None currently - all differences are transparent

## Testing Across Versions

### CI Matrix

Our CI tests these versions:
- b7035 (latest)
- b7000
- b6739 (breaking change)

### Local Testing

```bash
# Test with multiple versions
for v in b7035 b7000 b6739; do
    echo "Testing $v..."
    LLAMA_CPP_VERSION=$v cargo test || echo "FAILED: $v"
done
```

## Adding Version Support

When llama.cpp releases a new version with API changes:

### 1. Identify Changes

Check llama.cpp release notes and git diff:
```bash
git clone https://github.com/ggml-org/llama.cpp
cd llama.cpp
git diff b7035..b7100 -- include/llama.h
```

### 2. Add Version Detection

In `build.rs`:
```rust
fn emit_version_cfgs(version_num: u32) {
    if version_num >= 7100 {
        println!("cargo:rustc-cfg=llama_v7100");
        println!("cargo:rustc-cfg=llama_new_feature_name");
    }
}
```

### 3. Update Wrapper

Add conditional compilation:
```rust
#[cfg(llama_new_feature_name)]
pub fn use_new_api() { ... }

#[cfg(not(llama_new_feature_name))]
pub fn use_old_api() { ... }
```

### 4. Update Documentation

- Update `VERSION_DIFFERENCES.md`
- Update `API_VERSION_SUPPORT.md` (this file)
- Add notes to function docs

### 5. Test

```bash
LLAMA_CPP_VERSION=b7100 cargo test
LLAMA_CPP_VERSION=b7035 cargo test  # Ensure old still works
```

## User Guidelines

### For Library Users

**Recommended:**
```rust
// Use the safe wrapper - works across versions
let _backend = LlamaBackend::init();
let model = LlamaModel::load("model.gguf", ModelParams::default())?;
```

**Not Recommended:**
```rust
// Using raw FFI - version-dependent
unsafe {
    sys::llama_n_vocab(model)  // May not compile on b6739+
}
```

### Version Selection

**Production:**
- Pin to a specific tested version
- Test before upgrading

**Development:**
- Use latest for new features
- Test with multiple versions

## Summary

**Current Status:**
- ✅ All version differences handled transparently
- ✅ User code works across b6739+ without changes
- ✅ Compile-time version detection
- ✅ Zero runtime overhead

**Future:**
- When new features are added, document them here
- Use feature gates for genuinely new functionality
- Keep transparent handling for API changes
