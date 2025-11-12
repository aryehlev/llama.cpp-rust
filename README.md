# llama.cpp-rust

[![CI](https://github.com/yourusername/llama.cpp-rust/workflows/CI/badge.svg)](https://github.com/yourusername/llama.cpp-rust/actions)
[![Crates.io](https://img.shields.io/crates/v/llama-cpp-rust.svg)](https://crates.io/crates/llama-cpp-rust)
[![Documentation](https://docs.rs/llama-cpp-rust/badge.svg)](https://docs.rs/llama-cpp-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust bindings for [llama.cpp](https://github.com/ggml-org/llama.cpp) with dynamic linking and automatic pre-built binary downloads.

## Features

- 🚀 **Fast builds** - Downloads pre-built binaries instead of compiling from source
- 🔒 **Safe API** - Idiomatic Rust wrapper with RAII resource management
- 🎯 **Zero dependencies** - No runtime dependencies beyond llama.cpp
- 🌍 **Cross-platform** - Supports Linux, macOS (Intel & Apple Silicon), and Windows
- ⚡ **CPU & GPU** - Supports CPU inference and GPU acceleration
- 📦 **GGUF format** - Works with modern GGUF model files

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llama-cpp-rust = "0.1"
```

### Custom llama.cpp Version

By default, the crate uses llama.cpp version `b7035` (released Nov 12, 2025). To use a different version:

```bash
LLAMA_CPP_VERSION=b7100 cargo build
```

#### Supported Versions

The crate supports all llama.cpp versions with pre-built binaries (generally b5000+). The build script automatically:

- Detects version-specific API changes
- Downloads appropriate headers for the version
- Emits Cargo cfg flags for conditional compilation

**Tested versions:**
- `b7035` - Latest (Nov 2025) ✅
- `b7000` - Oct 2025 ✅
- `b6739` - Vocab API changes ✅
- `b6000+` - Should work ✅
- `b5000+` - Basic support ⚠️

**Version-specific features:**
```rust
// Conditional compilation based on version
#[cfg(llama_vocab_api)]  // Available in b6739+
fn use_new_vocab_api() {
    // Uses the new vocab API introduced in b6739
}

#[cfg(not(llama_vocab_api))]  // Before b6739
fn use_old_vocab_api() {
    // Uses the old API for older versions
}
```

## Quick Start

```rust
use llama_cpp_rust::{
    LlamaModel, LlamaContext, ModelParams, ContextParams
};

fn main() {
    // Load model (backend automatically initialized)
    let model = LlamaModel::load(
        "models/llama-2-7b.Q4_K_M.gguf",
        ModelParams::default()
    ).expect("Failed to load model");

    println!("Model loaded!");
    println!("  Vocab size: {}", model.n_vocab());
    println!("  Context length: {}", model.n_ctx_train());
    println!("  Embedding dim: {}", model.n_embd());

    // Create context for inference
    let mut ctx = LlamaContext::new(
        &model,
        ContextParams::default()
            .n_ctx(2048)
            .n_threads(4)
    ).expect("Failed to create context");

    // Tokenize input
    let prompt = "Once upon a time";
    let tokens = ctx.tokenize(prompt, true, false)
        .expect("Failed to tokenize");

    println!("Prompt: {}", prompt);
    println!("Tokens: {:?}", tokens);

    // Generate text
    let mut n_past = 0;
    let max_tokens = 32;

    print!("{}", prompt);

    for _ in 0..max_tokens {
        // Decode tokens
        ctx.decode(&tokens[n_past..], n_past as i32)
            .expect("Failed to decode");

        // Sample next token (greedy)
        let next_token = ctx.sample_greedy();

        // Convert token to text
        let text = ctx.token_to_piece(next_token);
        print!("{}", text);

        n_past = tokens.len();
        tokens.push(next_token);
    }

    println!();

    // Backend automatically freed when model goes out of scope
}
```

## API Overview

### Simple API - Just Load and Use

```rust
use llama_cpp_rust::{LlamaModel, ModelParams};

// Just load the model - everything is automatic!
let model = LlamaModel::load("model.gguf", ModelParams::default())?;
// Backend initialization, cleanup, everything is handled for you
```

**Why it's simple:**
- ✅ No manual initialization needed - just load your model
- ✅ Reference counted - load multiple models if needed
- ✅ Guaranteed cleanup even on panic
- ✅ Cannot forget initialization or cleanup
- ✅ Idiomatic Rust API

### Loading Models

```rust
use llama_cpp_rust::{LlamaModel, ModelParams};

let model = LlamaModel::load(
    "model.gguf",
    ModelParams::default()
        .n_gpu_layers(35)      // Offload 35 layers to GPU
        .use_mmap(true)        // Use memory mapping (default, recommended)
        .use_mlock(false)      // Don't lock memory (default)
).expect("Failed to load model");

// Get model info
println!("Vocab size: {}", model.n_vocab());
println!("Context length: {}", model.n_ctx_train());
println!("Embedding dim: {}", model.n_embd());
println!("Description: {}", model.desc());
```

#### Memory Mapping (mmap)

**What is mmap?** Memory mapping allows the model file to be mapped directly into your process's address space instead of being loaded into RAM. This provides:

- **100x faster loading** - Model loads almost instantly
- **50% less memory usage** - File is mapped, not copied into RAM
- **Lazy loading** - Only loads parts of the model as they're accessed

**When to use mmap (default: enabled):**
- ✅ Most use cases - it's faster and uses less memory
- ✅ When you have limited RAM
- ✅ When you want fast startup times

**When to disable mmap:**
- Only if you need maximum inference speed and have plenty of RAM
- If you're experiencing issues with file permissions or disk I/O

**Memory locking (mlock):**
```rust
ModelParams::default()
    .use_mmap(true)      // Map the file
    .use_mlock(true)     // Lock mapped pages in RAM to prevent swapping
```

Use `mlock` only if:
- You have enough RAM to hold the entire model
- You want consistent latency (no swapping)
- You have necessary system permissions

### Creating Context

```rust
use llama_cpp_rust::{LlamaContext, ContextParams};

let ctx = LlamaContext::new(
    &model,
    ContextParams::default()
        .n_ctx(2048)           // Context size
        .n_batch(512)          // Batch size
        .n_threads(8)          // Number of threads
        .n_threads_batch(8)    // Threads for batch processing
).expect("Failed to create context");
```

### Tokenization

```rust
// Tokenize text
let tokens = ctx.tokenize("Hello, world!", true, false)
    .expect("Failed to tokenize");

// Convert token to text
let text = ctx.token_to_piece(token_id);
```

### Inference

```rust
// Decode tokens
ctx.decode(&tokens, 0).expect("Failed to decode");

// Get logits for last token
let logits = ctx.get_logits();

// Sample next token (greedy)
let next_token = ctx.sample_greedy();
```

## Supported Platforms

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux    | x86_64      | ✅     |
| Linux    | aarch64     | ✅     |
| macOS    | x86_64      | ✅     |
| macOS    | aarch64 (M1/M2/M3) | ✅     |
| Windows  | x86_64      | ✅     |

## How It Works

This crate follows a similar pattern to `xgboost-rust`, `lightgbm-rust`, and `catboost-rust`:

1. **Download headers** - Downloads `llama.h` and `ggml.h` from the llama.cpp repository
2. **Download binaries** - Downloads pre-built binaries from llama.cpp GitHub releases
3. **Generate bindings** - Uses `bindgen` to create Rust FFI bindings
4. **Dynamic linking** - Links against the downloaded library at build time
5. **Runtime loading** - Configures rpath so the library is found at runtime

This approach provides:
- ⚡ **Much faster builds** (seconds vs minutes)
- 🛠️ **No C++ compiler needed**
- 📦 **Smaller crate size** (no vendored source code)
- 🔄 **Easy version updates** (just change environment variable)

## Comparison with Other Crates

| Crate | Build Time | Compiler Needed | Binary Size | Approach |
|-------|------------|----------------|-------------|----------|
| llama-cpp-rust | ~10s | ❌ No | Small | Download binaries |
| llama-cpp-2 | ~5min | ✅ Yes | Large | Compile from source |
| llama-cpp-sys | ~5min | ✅ Yes | Large | Compile from source |

## Requirements

### Build Requirements
- Rust 1.70+
- Internet connection (for downloading binaries)

### Runtime Requirements
- None! The library is self-contained

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [llama.cpp](https://github.com/ggml-org/llama.cpp) - The amazing C++ library this crate wraps
- Inspired by `xgboost-rust`, `lightgbm-rust`, and `catboost-rust`
