//! # llama.cpp-rust
//!
//! Rust bindings for llama.cpp with dynamic linking and pre-built binary downloads.
//!
//! This crate provides a safe, idiomatic Rust wrapper around the llama.cpp C API
//! for running Large Language Models locally with CPU or GPU acceleration.
//!
//! ## Features
//!
//! - Dynamic linking with automatic binary downloads
//! - Safe Rust API with RAII resource management
//! - Support for GGUF model format
//! - CPU and GPU inference
//! - Cross-platform support (Linux, macOS, Windows)
//!
//! ## Example
//!
//! ```no_run
//! use llama_cpp_rust::{
//!     LlamaModel, LlamaContext, ModelParams, ContextParams
//! };
//!
//! // Load model (backend automatically initialized)
//! let model = LlamaModel::load(
//!     "model.gguf",
//!     ModelParams::default()
//! ).unwrap();
//!
//! // Create context
//! let mut ctx = LlamaContext::new(
//!     &model,
//!     ContextParams::default().n_ctx(2048)
//! ).unwrap();
//!
//! // Tokenize input
//! let tokens = ctx.tokenize("Hello, world!", true, false).unwrap();
//!
//! // Run inference
//! ctx.decode(&tokens, 0).unwrap();
//!
//! // Sample next token
//! let next_token = ctx.sample_greedy();
//! let text = ctx.token_to_piece(next_token);
//! println!("Next token: {}", text);
//!
//! // Backend automatically freed when model is dropped
//! ```

mod error;
mod model;
mod sys;

pub use error::{LlamaError, LlamaResult};
pub use model::{ContextParams, LlamaContext, LlamaModel, ModelParams};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_params_default() {
        let params = ModelParams::default();
        // Test builder pattern
        let _params = params.n_gpu_layers(0).use_mmap(true).use_mlock(false);
    }

    #[test]
    fn test_context_params_default() {
        let params = ContextParams::default();
        // Test builder pattern
        let _params = params.n_ctx(512).n_batch(128).n_threads(4);
    }

    #[test]
    fn test_error_display() {
        let err = LlamaError::ModelLoadError("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    // Note: We can't test model loading without a real GGUF file,
    // so those tests would need to be integration tests with a test model
}
