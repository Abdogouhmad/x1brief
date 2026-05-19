use anyhow::{anyhow, Result};
use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_gemma3::ModelWeights as QGemma3;
use std::fs::File;
use tokenizers::Tokenizer;

pub struct X1Ai {
    model: QGemma3,
    tokenizer: Tokenizer,
    device: Device,
}

impl X1Ai {
    pub fn new(model_path: &str, tokenizer_path: &str) -> Result<Self> {
        let device = Device::Cpu;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        let mut file = File::open(model_path)
            .map_err(|e| anyhow!("Failed to open model file: {}", e))?;

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

    /// Summarize text using Gemma
    pub fn ai_sumup(&mut self, text: &str) -> Result<String> {
        let prompt = format!(
            "<start_of_turn>user\n\
             Summarize the following text in one short clear sentence:\n\n\
             {}\n\
             <end_of_turn>\n\
             <start_of_turn>model\n",
            text.trim()
        );

        // Tokenize prompt
        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let mut tokens = encoding.get_ids().to_vec();

        // Sampling configuration
        let mut logits_processor =
            LogitsProcessor::new(1337, Some(0.3), Some(0.9));

        let repeat_penalty = 1.1f32;
        let repeat_last_n = 64usize;

        let mut output_tokens = Vec::new();
        let mut pos = 0;

        for i in 0..100 {
            let input_tokens = if i == 0 {
                &tokens[..]
            } else {
                &tokens[tokens.len() - 1..]
            };

            let input = Tensor::new(input_tokens, &self.device)?
                .unsqueeze(0)?;

            let logits = self.model.forward(&input, pos)?;

            let mut logits = logits.squeeze(0)?;

            // Get logits for last generated token
            if logits.dims().len() == 2 {
                logits = logits.get(logits.dim(0)? - 1)?;
            }

            // Apply repetition penalty
            let start_at = tokens.len().saturating_sub(repeat_last_n);

            logits = candle_transformers::utils::apply_repeat_penalty(
                &logits,
                repeat_penalty,
                &tokens[start_at..],
            )?;

            pos += input_tokens.len();

            let next_token = logits_processor.sample(&logits)?;

            // Stop generation
            if next_token
                == self
                    .tokenizer
                    .token_to_id("<end_of_turn>")
                    .unwrap_or(107)
                || next_token
                    == self
                        .tokenizer
                        .token_to_id("<eos>")
                        .unwrap_or(1)
            {
                break;
            }

            output_tokens.push(next_token);
            tokens.push(next_token);
        }

        // Decode generated tokens
        let summary = self
            .tokenizer
            .decode(&output_tokens, true)
            .map_err(|e| anyhow!("Decoding failed: {}", e))?;

        Ok(summary.trim().to_string())
    }
}
