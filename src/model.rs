use crate::error::{LlamaError, LlamaResult};
use crate::sys;
use once_cell::sync::Lazy;
use std::ffi::CString;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Backend lifecycle guard
///
/// When created, initializes the llama.cpp backend.
/// When dropped, frees the llama.cpp backend.
struct BackendGuard;

impl BackendGuard {
    fn new() -> Self {
        unsafe {
            sys::llama_backend_init();
        }
        BackendGuard
    }
}

impl Drop for BackendGuard {
    fn drop(&mut self) {
        unsafe {
            sys::llama_backend_free();
        }
    }
}

/// Global backend instance
///
/// Why Mutex<Option<Arc<BackendGuard>>>?
/// - Mutex: Thread-safe creation/access
/// - Option: Allows taking ownership (setting to None) when last Arc drops
/// - Arc: Reference counting - backend freed when count reaches 0
///
/// This ensures the backend is freed when the last model is dropped,
/// unlike `Lazy<Arc<T>>` which would hold a reference forever.
static BACKEND: Lazy<Mutex<Option<Arc<BackendGuard>>>> = Lazy::new(|| Mutex::new(None));

/// A llama.cpp model
pub struct LlamaModel {
    model: *mut sys::llama_model,
    _backend: Arc<BackendGuard>,
}

impl LlamaModel {
    /// Load a model from a file
    ///
    /// The backend is automatically initialized when the first model is loaded,
    /// and automatically freed when the last model is dropped.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GGUF model file
    /// * `params` - Model parameters (use `ModelParams::default()` for defaults)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use llama_cpp_rust::{LlamaModel, ModelParams};
    ///
    /// // Backend is automatically initialized here
    /// let model = LlamaModel::load("model.gguf", ModelParams::default()).unwrap();
    /// // Backend is automatically freed when model is dropped
    /// ```
    pub fn load<P: AsRef<Path>>(path: P, params: ModelParams) -> LlamaResult<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| LlamaError::InvalidParameter("Invalid path".to_string()))?;
        let path_c = CString::new(path_str)
            .map_err(|e| LlamaError::InvalidParameter(format!("Invalid path: {}", e)))?;

        // Get or create the backend guard
        let mut backend_opt = BACKEND.lock().unwrap();
        let backend = match &*backend_opt {
            Some(guard) => Arc::clone(guard),
            None => {
                let guard = Arc::new(BackendGuard::new());
                *backend_opt = Some(Arc::clone(&guard));
                guard
            }
        };
        drop(backend_opt); // Release the lock

        let model = unsafe { sys::llama_load_model_from_file(path_c.as_ptr(), params.inner) };

        let model = LlamaError::check_null(model, "llama_load_model_from_file")?;

        Ok(LlamaModel {
            model,
            _backend: backend,
        })
    }

    /// Get the number of vocabulary tokens
    pub fn n_vocab(&self) -> i32 {
        #[cfg(llama_vocab_api)]
        {
            let vocab = unsafe { sys::llama_model_get_vocab(self.model) };
            unsafe { sys::llama_n_vocab(vocab) }
        }

        #[cfg(not(llama_vocab_api))]
        {
            unsafe { sys::llama_n_vocab(self.model) }
        }
    }

    /// Get the model's context length
    pub fn n_ctx_train(&self) -> i32 {
        unsafe { sys::llama_n_ctx_train(self.model) }
    }

    /// Get the model's embedding dimension
    pub fn n_embd(&self) -> i32 {
        unsafe { sys::llama_n_embd(self.model) }
    }

    /// Get model description
    pub fn desc(&self) -> String {
        let mut buf = vec![0u8; 128];
        let len =
            unsafe { sys::llama_model_desc(self.model, buf.as_mut_ptr() as *mut i8, buf.len()) };
        buf.truncate(len as usize);
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl Drop for LlamaModel {
    fn drop(&mut self) {
        unsafe {
            sys::llama_free_model(self.model);
        }
        // Backend is automatically freed when _backend Arc is dropped
    }
}

// NOTE: LlamaModel is NOT thread-safe. The underlying llama.cpp C API
// is not designed for concurrent access from multiple threads.
// If you need to use a model from multiple threads, wrap it in Arc<Mutex<LlamaModel>>
// or use separate model instances per thread.

/// Model parameters for loading a model
pub struct ModelParams {
    inner: sys::llama_model_params,
}

impl ModelParams {
    /// Create model parameters with custom settings
    pub fn new() -> Self {
        Self {
            inner: unsafe { sys::llama_model_default_params() },
        }
    }

    /// Set the number of GPU layers to offload
    pub fn n_gpu_layers(mut self, n: i32) -> Self {
        self.inner.n_gpu_layers = n;
        self
    }

    /// Set whether to use memory mapping (mmap)
    ///
    /// When enabled (default), the model file is memory-mapped instead of loaded into RAM.
    /// This provides significant benefits:
    /// - **100x faster loading** - Model loads almost instantly
    /// - **50% less memory** - File is mapped, not copied into RAM
    /// - **Lazy loading** - Only loads model parts as they're accessed
    ///
    /// This is the recommended approach for most use cases. Only disable if you need
    /// the model to be fully resident in RAM (e.g., for maximum inference speed on
    /// systems with plenty of memory).
    pub fn use_mmap(mut self, use_mmap: bool) -> Self {
        self.inner.use_mmap = use_mmap;
        self
    }

    /// Set whether to use memory locking (mlock)
    ///
    /// When enabled, locks the model in RAM to prevent it from being swapped to disk.
    /// This can improve performance but requires sufficient RAM and may require elevated
    /// privileges on some systems.
    ///
    /// Only enable this if:
    /// - You have enough RAM to hold the entire model
    /// - You want to prevent swapping for consistent latency
    /// - You have the necessary system permissions
    pub fn use_mlock(mut self, use_mlock: bool) -> Self {
        self.inner.use_mlock = use_mlock;
        self
    }
}

impl Default for ModelParams {
    fn default() -> Self {
        Self::new()
    }
}

/// A llama.cpp context for inference
pub struct LlamaContext {
    ctx: *mut sys::llama_context,
    model: *mut sys::llama_model,
}

impl LlamaContext {
    /// Create a new context from a model
    ///
    /// # Arguments
    ///
    /// * `model` - The model to create context for
    /// * `params` - Context parameters (use `ContextParams::default()` for defaults)
    pub fn new(model: &LlamaModel, params: ContextParams) -> LlamaResult<Self> {
        let ctx = unsafe { sys::llama_new_context_with_model(model.model, params.inner) };

        let ctx = LlamaError::check_null(ctx, "llama_new_context_with_model")?;

        Ok(LlamaContext {
            ctx,
            model: model.model,
        })
    }

    /// Tokenize text
    ///
    /// # Arguments
    ///
    /// * `text` - Text to tokenize
    /// * `add_special` - Whether to add special tokens (BOS, etc.)
    /// * `parse_special` - Whether to parse special tokens
    pub fn tokenize(
        &self,
        text: &str,
        add_special: bool,
        parse_special: bool,
    ) -> LlamaResult<Vec<i32>> {
        let text_c = CString::new(text)
            .map_err(|e| LlamaError::TokenizationError(format!("Invalid text: {}", e)))?;

        let mut tokens = vec![0i32; text.len() + 16]; // Allocate with extra space

        #[cfg(llama_vocab_api)]
        let n_tokens = unsafe {
            let vocab = sys::llama_model_get_vocab(self.model);
            sys::llama_tokenize(
                vocab,
                text_c.as_ptr(),
                text.len() as i32,
                tokens.as_mut_ptr(),
                tokens.len() as i32,
                add_special,
                parse_special,
            )
        };

        #[cfg(not(llama_vocab_api))]
        let n_tokens = unsafe {
            sys::llama_tokenize(
                self.model,
                text_c.as_ptr(),
                text.len() as i32,
                tokens.as_mut_ptr(),
                tokens.len() as i32,
                add_special,
                parse_special,
            )
        };

        if n_tokens < 0 {
            return Err(LlamaError::TokenizationError(format!(
                "Tokenization failed, need {} tokens but buffer too small",
                -n_tokens
            )));
        }

        tokens.truncate(n_tokens as usize);
        Ok(tokens)
    }

    /// Decode tokens to text
    pub fn token_to_piece(&self, token: i32) -> String {
        let mut buf = vec![0u8; 32];

        #[cfg(llama_vocab_api)]
        let len = unsafe {
            let vocab = sys::llama_model_get_vocab(self.model);
            sys::llama_token_to_piece(
                vocab,
                token,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as i32,
                0,
                true,
            )
        };

        #[cfg(not(llama_vocab_api))]
        let len = unsafe {
            sys::llama_token_to_piece(
                self.model,
                token,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as i32,
                0,
                true,
            )
        };

        if len < 0 {
            return String::new();
        }

        buf.truncate(len as usize);
        String::from_utf8_lossy(&buf).to_string()
    }

    /// Evaluate (decode) tokens
    ///
    /// # Arguments
    ///
    /// * `tokens` - Tokens to decode
    /// * `n_past` - Number of tokens already processed
    pub fn decode(&mut self, tokens: &[i32], n_past: i32) -> LlamaResult<()> {
        let mut batch = unsafe { sys::llama_batch_init(tokens.len() as i32, 0, 1) };

        batch.n_tokens = tokens.len() as i32;

        for (i, &token) in tokens.iter().enumerate() {
            unsafe {
                *batch.token.add(i) = token;
                *batch.pos.add(i) = n_past + i as i32;
                *batch.n_seq_id.add(i) = 1;
                *(*batch.seq_id.add(i)).add(0) = 0;
                *batch.logits.add(i) = if i == tokens.len() - 1 { 1 } else { 0 };
            }
        }

        let result = unsafe { sys::llama_decode(self.ctx, batch) };

        unsafe { sys::llama_batch_free(batch) };

        if result != 0 {
            return Err(LlamaError::InferenceError(format!(
                "llama_decode failed with code {}",
                result
            )));
        }

        Ok(())
    }

    /// Get logits for the last token
    pub fn get_logits(&self) -> &[f32] {
        #[cfg(llama_vocab_api)]
        let n_vocab = unsafe {
            let vocab = sys::llama_model_get_vocab(self.model);
            sys::llama_n_vocab(vocab)
        };

        #[cfg(not(llama_vocab_api))]
        let n_vocab = unsafe { sys::llama_n_vocab(self.model) };

        let logits_ptr = unsafe { sys::llama_get_logits_ith(self.ctx, -1) };

        if logits_ptr.is_null() {
            return &[];
        }

        unsafe { std::slice::from_raw_parts(logits_ptr, n_vocab as usize) }
    }

    /// Sample a token from logits using greedy sampling
    pub fn sample_greedy(&self) -> i32 {
        let logits = self.get_logits();
        if logits.is_empty() {
            return 0;
        }

        logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx as i32)
            .unwrap_or(0)
    }

    /// Get the context size
    pub fn n_ctx(&self) -> u32 {
        unsafe { sys::llama_n_ctx(self.ctx) }
    }
}

impl Drop for LlamaContext {
    fn drop(&mut self) {
        unsafe {
            sys::llama_free(self.ctx);
        }
    }
}

/// Context parameters for creating inference context
pub struct ContextParams {
    inner: sys::llama_context_params,
}

impl ContextParams {
    /// Create context parameters with custom settings
    pub fn new() -> Self {
        Self {
            inner: unsafe { sys::llama_context_default_params() },
        }
    }

    /// Set the context size (max number of tokens)
    pub fn n_ctx(mut self, n: u32) -> Self {
        self.inner.n_ctx = n;
        self
    }

    /// Set the batch size for prompt processing
    pub fn n_batch(mut self, n: u32) -> Self {
        self.inner.n_batch = n;
        self
    }

    /// Set the number of threads to use
    pub fn n_threads(mut self, n: u32) -> Self {
        self.inner.n_threads = n as i32;
        self
    }

    /// Set the number of threads to use for batch processing
    pub fn n_threads_batch(mut self, n: u32) -> Self {
        self.inner.n_threads_batch = n as i32;
        self
    }
}

impl Default for ContextParams {
    fn default() -> Self {
        Self::new()
    }
}
