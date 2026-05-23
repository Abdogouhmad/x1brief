use anyhow::{Result, bail};
use candle_pipelines::text_generation::{
    Gemma3, Llama3_2, Qwen3, TextGenerationPipeline, TextGenerationPipelineBuilder,
};
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ModelFamily {
    Gemma,
    Qwen,
    Llama,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ModelSize {
    Small,
    Large,
}

enum Pipeline {
    Qwen(TextGenerationPipeline<Qwen3>),
    Gemma(TextGenerationPipeline<Gemma3>),
    Llama(TextGenerationPipeline<Llama3_2>),
}

pub struct X1Ai {
    pipeline: Option<Pipeline>,
    family: ModelFamily,
    size: ModelSize,
}

impl X1Ai {
    pub fn new(family: ModelFamily, size: ModelSize) -> Result<Self> {
        Ok(Self {
            pipeline: None,
            family,
            size,
        })
    }

    fn set_pipeline(&mut self) -> Result<()> {
        if self.pipeline.is_some() {
            return Ok(());
        }

        let pipeline = match (self.family, self.size) {
            // ---------------- GEMMA ----------------
            (ModelFamily::Gemma, ModelSize::Small) => {
                Pipeline::Gemma(TextGenerationPipelineBuilder::gemma3(Gemma3::Size12B).build()?)
            }

            (ModelFamily::Gemma, ModelSize::Large) => {
                Pipeline::Gemma(TextGenerationPipelineBuilder::gemma3(Gemma3::Size27B).build()?)
            }

            // ---------------- QWEN ----------------
            (ModelFamily::Qwen, ModelSize::Small) => {
                Pipeline::Qwen(TextGenerationPipelineBuilder::qwen3(Qwen3::Size4B).build()?)
            }

            (ModelFamily::Qwen, ModelSize::Large) => {
                Pipeline::Qwen(TextGenerationPipelineBuilder::qwen3(Qwen3::Size32B).build()?)
            }

            // ---------------- LLAMA ----------------
            (ModelFamily::Llama, ModelSize::Small) => {
                Pipeline::Llama(TextGenerationPipelineBuilder::llama3_2(Llama3_2::Size1B).build()?)
            }

            (ModelFamily::Llama, ModelSize::Large) => {
                Pipeline::Llama(TextGenerationPipelineBuilder::llama3_2(Llama3_2::Size3B).build()?)
            }
        };

        self.pipeline = Some(pipeline);

        Ok(())
    }

    pub fn generate(&mut self, prompt: &str) -> Result<String> {
        if self.pipeline.is_none() {
            self.set_pipeline()?;
        }

        let Some(pipeline) = self.pipeline.as_mut() else {
            bail!("Pipeline failed to initialize");
        };

        let output = match pipeline {
            Pipeline::Gemma(p) => p.run(prompt)?,
            Pipeline::Qwen(p) => p.run(prompt)?,
            Pipeline::Llama(p) => p.run(prompt)?,
        };

        Ok(output.text)
    }
}
