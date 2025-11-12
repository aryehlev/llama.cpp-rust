use llama_cpp_rust::{ContextParams, LlamaContext, LlamaModel, ModelParams};
use std::env;

fn main() {
    // Backend is automatically initialized when loading the model

    // Get model path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <model.gguf> [prompt]", args[0]);
        eprintln!(
            "Example: {} models/llama-2-7b.Q4_K_M.gguf \"Once upon a time\"",
            args[0]
        );
        std::process::exit(1);
    }

    let model_path = &args[1];
    let prompt = if args.len() > 2 {
        args[2].as_str()
    } else {
        "Hello, how are you?"
    };

    // Load model
    println!("Loading model from: {}", model_path);
    let model = match LlamaModel::load(model_path, ModelParams::default()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load model: {}", e);
            std::process::exit(1);
        }
    };

    println!("Model loaded successfully!");
    println!("  Description: {}", model.desc());
    println!("  Vocabulary size: {}", model.n_vocab());
    println!("  Training context length: {}", model.n_ctx_train());
    println!("  Embedding dimension: {}", model.n_embd());
    println!();

    // Create context
    let mut ctx = match LlamaContext::new(
        &model,
        ContextParams::default()
            .n_ctx(2048)
            .n_threads(4)
            .n_threads_batch(4),
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create context: {}", e);
            std::process::exit(1);
        }
    };

    println!("Context created successfully!");
    println!("  Context size: {}", ctx.n_ctx());
    println!();

    // Tokenize prompt
    let mut tokens = match ctx.tokenize(prompt, true, false) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to tokenize: {}", e);
            std::process::exit(1);
        }
    };

    println!("Prompt: {}", prompt);
    println!("Tokenized into {} tokens", tokens.len());
    println!();

    // Generate text
    print!("Generated: {}", prompt);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let max_tokens = 100;
    let mut n_past = 0;

    for _ in 0..max_tokens {
        // Decode current tokens
        if let Err(e) = ctx.decode(&tokens[n_past..], n_past as i32) {
            eprintln!("\nFailed to decode: {}", e);
            break;
        }

        // Sample next token (greedy sampling)
        let next_token = ctx.sample_greedy();

        // Check for end of text
        if next_token == 0 || next_token == 2 {
            // Common EOS tokens
            break;
        }

        // Convert token to text and print
        let text = ctx.token_to_piece(next_token);
        print!("{}", text);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        // Add token to history
        tokens.push(next_token);
        n_past = tokens.len() - 1;

        // Check context limit
        if tokens.len() >= ctx.n_ctx() as usize {
            println!("\n\nReached context limit!");
            break;
        }
    }

    println!("\n\nGenerated {} tokens total", tokens.len());

    // Backend is automatically cleaned up when model goes out of scope
}
