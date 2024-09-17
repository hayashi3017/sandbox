use either::Either;
use indexmap::IndexMap;
use std::{num::NonZero, sync::Arc};
use tokio::sync::mpsc::channel;

use mistralrs::{
    Constraint, DefaultSchedulerMethod, Device, DeviceMapMetadata, GGUFLoaderBuilder, GGUFSpecificConfig, MemoryGpuConfig, MistralRs, MistralRsBuilder, ModelDType, NormalRequest, PagedAttentionConfig, Request, RequestMessage, ResponseOk, Result, SamplingParams, SchedulerConfig, TokenSource
};

/// Gets the best device, cpu, cuda if compiled with CUDA
pub(crate) fn best_device() -> Result<Device> {
    #[cfg(not(feature = "metal"))]
    {
        Device::cuda_if_available(0)
    }
    #[cfg(feature = "metal")]
    {
        Device::new_metal(0)
    }
}

async fn setup() -> anyhow::Result<Arc<MistralRs>> {
    // Select a Mistral model
    // We do not use any files from HF servers here, and instead load the
    // chat template from the specified file, and the tokenizer and model from a
    // local GGUF file at the path `.`
    let loader = GGUFLoaderBuilder::new(
        // chat_templateに何を指定すればよいのか不明。
        // tokenizer_config.jsonはget_tokenizers_json.pyによって取得できた
        // tokenizer_config.jsonからchat_templateに変換する？必要があるのか、方法が不明。どちらも別々に必要そうだが、HFにはない
        // https://huggingface.co/elyza/ELYZA-japanese-Llama-2-7b-instruct
        // おそらくエラーが起こらないようなJIJAテンプレートを定義する必要がある、mistral.jsonは上手くいくパターン
        Some("chat_templates/mistral.json".to_string()),
        // Some("chat_templates/llama2.json".to_string()),
        // Some("chat_templates/chatml.json".to_string()),
        // Some("chat_templates/default.json".to_string()),
        // Some("tokenizer_config.json".to_string()),
        // None,
        Some("elyza/ELYZA-japanese-Llama-2-7b-instruct".to_string()),
        "mmnga/ELYZA-japanese-Llama-2-7b-instruct-gguf".to_string(),
        vec!["ELYZA-japanese-Llama-2-7b-instruct-q4_K_M.gguf".to_string()],
        GGUFSpecificConfig {
            prompt_batchsize: None,
            topology: None,
        },
    )
    .build();
    let cache_config = Some(PagedAttentionConfig::new(
        Some(32),
        512,
        MemoryGpuConfig::Utilization(0.9), // NOTE(EricLBuehler): default is to use 90% of memory
    )?);
    // Load, into a Pipeline
    let pipeline = loader.load_model_from_hf(
        None,
        TokenSource::CacheToken,
        &ModelDType::Auto,
        &best_device()?,
        false,
        DeviceMapMetadata::dummy(),
        None,
        cache_config, // No PagedAttention.
    )?;
    let scheduler_config = if cache_config.is_some() {
        // Handle case where we may have device mapping
        if let Some(ref cache_config) = pipeline.lock().await.get_metadata().cache_config {
            SchedulerConfig::PagedAttentionMeta {
                max_num_seqs: 16,
                config: cache_config.clone(),
            }
        } else {
            SchedulerConfig::DefaultScheduler {
                method: DefaultSchedulerMethod::Fixed(NonZero::new(16).unwrap()),
            }
        }
    } else {
        SchedulerConfig::DefaultScheduler {
            method: DefaultSchedulerMethod::Fixed(NonZero::new(16).unwrap()),
        }
    };
    // Create the MistralRs, which is a runner
    Ok(MistralRsBuilder::new(pipeline, scheduler_config).build())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mistralrs = setup().await?;

    let (tx, mut rx) = channel(10_000);
    let request = Request::Normal(NormalRequest {
        messages: RequestMessage::Chat(vec![IndexMap::from([
            ("role".to_string(), Either::Left("user".to_string())),
            ("content".to_string(), Either::Left("日本の伝統料理について教えて".to_string())),
        ])]),
        sampling_params: SamplingParams::default(),
        response: tx,
        return_logprobs: false,
        is_streaming: false,
        id: 0,
        constraint: Constraint::None,
        suffix: None,
        adapters: None,
        tools: None,
        tool_choice: None,
        logits_processors: None,
    });
    mistralrs.get_sender()?.send(request).await?;

    let response = rx.recv().await.unwrap().as_result().unwrap();
    match response {
        ResponseOk::Done(c) => println!(
            "Text: {}, Prompt T/s: {}, Completion T/s: {}",
            c.choices[0].message.content.as_ref().unwrap(),
            c.usage.avg_prompt_tok_per_sec,
            c.usage.avg_compl_tok_per_sec
        ),
        _ => unreachable!(),
    }
    Ok(())
}
