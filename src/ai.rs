use anyhow::{Result, anyhow};
use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_gemma3::ModelWeights as QGemma3;
use std::fs::File;
use tokenizers::Tokenizer;

pub struct Ai {
    model: QGemma3,
    tokenizer: Tokenizer,
    device: Device,
}

impl Ai {
    pub fn new(model_path: &str, tokenizer_path: &str) -> Result<Self> {
        let device = Device::Cpu;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        let mut file = File::open(model_path)?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| anyhow!("Failed to read GGUF file: {}", e))?;

        let model = QGemma3::from_gguf(content, &mut file, &device)
            .map_err(|e| anyhow!("Failed to load model from GGUF: {}", e))?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }
    pub async fn summarize(&mut self, text: &str) -> Result<String> {
        // 1. Refined Prompt: Use clear instruction-tuned tags
        // Small models like 270M work better when the instruction is very close to the text.
        let prompt = format!(
            "<start_of_turn>user\nSummarize the following text in one short and clear:\n\n{}\n<end_of_turn>\n<start_of_turn>model\n",
            text.trim()
        );

        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;
        let mut tokens = tokens.get_ids().to_vec();

        // 2. Stronger Sampling: Lower temperature (0.3) and added repetition penalty (1.1)
        // Most candle LogitsProcessors allow for a repetition penalty to prevent loops.
        let mut logits_processor = LogitsProcessor::new(1337, Some(0.3), Some(0.9));
        let repeat_penalty = 1.1f32;
        let repeat_last_n = 64; // Look back at the last 64 tokens for repeats

        let mut output_tokens = Vec::new();
        let mut pos = 0;

        for i in 0..100 {
            // Reduced to 100 to keep it "concise"
            let input_tokens = if i == 0 {
                &tokens[..]
            } else {
                &tokens[tokens.len() - 1..]
            };

            let input = Tensor::new(input_tokens, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, pos)?;
            let mut logits = logits.squeeze(0)?;

            // Apply logic to the last token's logits
            if logits.dims().len() == 2 {
                logits = logits.get(logits.dim(0)? - 1)?;
            }

            // --- Manual Repetition Penalty ---
            // If your candle version doesn't have it built-in, this prevents the "loop"
            let start_at = tokens.len().saturating_sub(repeat_last_n);
            candle_transformers::utils::apply_repeat_penalty(
                &logits,
                repeat_penalty,
                &tokens[start_at..],
            )?;

            pos += input_tokens.len();

            let next_token = logits_processor.sample(&logits)?;

            // Stop if we hit the end of turn or end of stream
            if next_token == self.tokenizer.token_to_id("<end_of_turn>").unwrap_or(107)
                || next_token == self.tokenizer.token_to_id("<eos>").unwrap_or(1)
            {
                break;
            }

            output_tokens.push(next_token);
            tokens.push(next_token);
        }

        let summary = self
            .tokenizer
            .decode(&output_tokens, true)
            .map_err(|e| anyhow!("Decoding failed: {}", e))?;

        Ok(summary.trim().to_string())
    }
}
