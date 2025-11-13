use std::error::Error;
use std::fmt;

/// Result type for llama.cpp operations
pub type LlamaResult<T> = std::result::Result<T, LlamaError>;

/// Error type for llama.cpp operations
#[derive(Debug, Clone)]
pub enum LlamaError {
    /// Error loading model
    ModelLoadError(String),
    /// Error creating context
    ContextCreationError(String),
    /// Error during tokenization
    TokenizationError(String),
    /// Error during inference
    InferenceError(String),
    /// Null pointer error
    NullPointerError(String),
    /// Invalid parameter
    InvalidParameter(String),
    /// Other error
    Other(String),
}

impl fmt::Display for LlamaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlamaError::ModelLoadError(msg) => write!(f, "Model load error: {}", msg),
            LlamaError::ContextCreationError(msg) => write!(f, "Context creation error: {}", msg),
            LlamaError::TokenizationError(msg) => write!(f, "Tokenization error: {}", msg),
            LlamaError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            LlamaError::NullPointerError(msg) => write!(f, "Null pointer error: {}", msg),
            LlamaError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            LlamaError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for LlamaError {}

impl LlamaError {
    /// Check if a pointer is null and return an error if so
    pub fn check_null<T>(ptr: *mut T, context: &str) -> LlamaResult<*mut T> {
        if ptr.is_null() {
            Err(LlamaError::NullPointerError(format!(
                "Null pointer in {}",
                context
            )))
        } else {
            Ok(ptr)
        }
    }
}
