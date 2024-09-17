use anyhow::{Context as _, Result};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{self, Method},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use either::Either;
use indexmap::IndexMap;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, env, error::Error, num::NonZero, ops::Deref, pin::Pin, sync::Arc, task::{Context, Poll}, time::Duration
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower_http::cors::{AllowOrigin, CorsLayer};

use mistralrs::{
    ChatCompletionResponse, Constraint, DefaultSchedulerMethod, Device, DeviceMapMetadata,
    DrySamplingParams, IsqType, MemoryGpuConfig, MistralRs, MistralRsBuilder, ModelDType,
    NormalLoaderBuilder, NormalRequest, NormalSpecificConfig, PagedAttentionConfig, Request,
    RequestMessage, Response, SamplingParams, SchedulerConfig, StopTokens as InternalStopTokens,
    TokenSource, Tool, ToolChoice,
};

// NOTE(EricLBuehler): Accept up to 50mb input
const N_INPUT_SIZE: usize = 50;
const MB_TO_B: usize = 1024 * 1024; // 1024 kb in a mb

/// Gets the best device, cpu, cuda if compiled with CUDA
pub(crate) fn best_device() -> Result<Device> {
    #[cfg(not(feature = "metal"))]
    {
        Ok(Device::cuda_if_available(0)?)
    }
    #[cfg(feature = "metal")]
    {
        Device::new_metal(0)
    }
}

async fn setup() -> anyhow::Result<Arc<MistralRs>> {
    // Select a Mistral model
    let loader = NormalLoaderBuilder::new(
        NormalSpecificConfig {
            use_flash_attn: false,
            prompt_batchsize: None,
            topology: None,
            organization: Default::default(),
        },
        None,
        None,
        Some("elyza/ELYZA-japanese-Llama-2-7b-instruct".to_string()),
    )
    .build(None)?;
    // Load, into a Pipeline
    let cache_config = Some(PagedAttentionConfig::new(
        Some(32),
        512,
        MemoryGpuConfig::Utilization(0.9), // NOTE(EricLBuehler): default is to use 90% of memory
    )?);

    let pipeline = loader.load_model_from_hf(
        None,
        TokenSource::CacheToken,
        &ModelDType::Auto,
        &best_device()?,
        false,
        DeviceMapMetadata::dummy(),
        Some(IsqType::Q4K), // In-situ quantize the model into q4k
        cache_config,       // No PagedAttention.
    )?;
    println!("Model loaded.");
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

    // let (tx, mut rx) = channel(10_000);
    // let request = Request::Normal(NormalRequest {
    //     messages: RequestMessage::Chat(vec![IndexMap::from([
    //         ("role".to_string(), Either::Left("user".to_string())),
    //         ("content".to_string(), Either::Left("Hello!".to_string())),
    //     ])]),
    //     sampling_params: SamplingParams::default(),
    //     response: tx,
    //     return_logprobs: false,
    //     is_streaming: false,
    //     id: 0,
    //     constraint: Constraint::None,
    //     suffix: None,
    //     adapters: None,
    //     tools: None,
    //     tool_choice: None,
    //     logits_processors: None,
    // });
    // mistralrs.get_sender()?.blocking_send(request)?;

    // let response = rx.blocking_recv().unwrap().as_result().unwrap();
    // match response {
    //     ResponseOk::Done(c) => println!(
    //         "Text: {}, Prompt T/s: {}, Completion T/s: {}",
    //         c.choices[0].message.content.as_ref().unwrap(),
    //         c.usage.avg_prompt_tok_per_sec,
    //         c.usage.avg_compl_tok_per_sec
    //     ),
    //     _ => unreachable!(),
    // }

    let app = get_router(mistralrs);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:1234")).await?;
    println!("Serving on localhost:1234.");
    axum::serve(listener, app).await?;
    Ok(())
}

fn get_router(state: Arc<MistralRs>) -> Router {
    let allow_origin = AllowOrigin::any();
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .allow_origin(allow_origin);

    Router::new()
        .route("/v1/chat/completions", post(chatcompletions))
        .route("/health", get(health))
        .layer(cors_layer)
        .layer(DefaultBodyLimit::max(N_INPUT_SIZE * MB_TO_B))
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}

pub async fn chatcompletions(
    State(state): State<Arc<MistralRs>>,
    Json(oairequest): Json<ChatCompletionRequest>,
) -> ChatCompletionResponder {
    let (tx, mut rx) = channel(10_000);
    let (request, is_streaming) = match parse_request(oairequest, state.clone(), tx).await {
        Ok(x) => x,
        Err(e) => {
            let e = anyhow::Error::msg(e.to_string());
            MistralRs::maybe_log_error(state, &*e);
            return ChatCompletionResponder::InternalError(e.into());
        }
    };
    let sender = state.get_sender().unwrap();

    if let Err(e) = sender.send(request).await {
        let e = anyhow::Error::msg(e.to_string());
        MistralRs::maybe_log_error(state, &*e);
        return ChatCompletionResponder::InternalError(e.into());
    }

    if is_streaming {
        let streamer = Streamer {
            rx,
            is_done: false,
            state,
        };

        ChatCompletionResponder::Sse(
            Sse::new(streamer).keep_alive(
                KeepAlive::new()
                    .interval(Duration::from_millis(
                        env::var("KEEP_ALIVE_INTERVAL")
                            .map(|val| val.parse::<u64>().unwrap_or(1000))
                            .unwrap_or(1000),
                    ))
                    .text("keep-alive-text"),
            ),
        )
    } else {
        let response = match rx.recv().await {
            Some(response) => response,
            None => {
                let e = anyhow::Error::msg("No response received from the model.");
                MistralRs::maybe_log_error(state, &*e);
                return ChatCompletionResponder::InternalError(e.into());
            }
        };

        match response {
            Response::InternalError(e) => {
                MistralRs::maybe_log_error(state, &*e);
                ChatCompletionResponder::InternalError(e)
            }
            Response::ModelError(msg, response) => {
                MistralRs::maybe_log_error(state.clone(), &ModelErrorMessage(msg.to_string()));
                MistralRs::maybe_log_response(state, &response);
                ChatCompletionResponder::ModelError(msg, response)
            }
            Response::ValidationError(e) => ChatCompletionResponder::ValidationError(e),
            Response::Done(response) => {
                MistralRs::maybe_log_response(state, &response);
                ChatCompletionResponder::Json(response)
            }
            Response::Chunk(_) => unreachable!(),
            Response::CompletionDone(_) => unreachable!(),
            Response::CompletionModelError(_, _) => unreachable!(),
            Response::CompletionChunk(_) => unreachable!(),
        }
    }
}

async fn parse_request(
    oairequest: ChatCompletionRequest,
    state: Arc<MistralRs>,
    tx: Sender<Response>,
) -> Result<(Request, bool)> {
    let repr = serde_json::to_string(&oairequest).expect("Serialization of request failed.");
    MistralRs::maybe_log_request(state.clone(), repr);

    let stop_toks = match oairequest.stop_seqs {
        Some(StopTokens::Multi(m)) => Some(InternalStopTokens::Seqs(m)),
        Some(StopTokens::Single(s)) => Some(InternalStopTokens::Seqs(vec![s])),
        None => None,
    };
    let messages = match oairequest.messages {
        Either::Left(req_messages) => {
            let mut messages = Vec::new();
            let mut image_urls = Vec::new();
            for message in req_messages {
                match message.content.0 {
                    Either::Left(content) => {
                        let mut message_map: IndexMap<
                            String,
                            Either<String, Vec<IndexMap<String, String>>>,
                        > = IndexMap::new();
                        message_map.insert("role".to_string(), Either::Left(message.role));
                        message_map
                            .insert("content".to_string(), Either::Left(content.to_string()));
                        messages.push(message_map);
                    }
                    Either::Right(image_messages) => {
                        if image_messages.len() != 2 {
                            anyhow::bail!(
                                "Expected 2 items for the content of a message with an image."
                            );
                        }
                        if message.role != "user" {
                            anyhow::bail!(
                                "Role for an image message must be `user`, but it is {}",
                                message.role
                            );
                        }

                        let mut items = Vec::new();
                        for image_message in &image_messages {
                            if image_message.len() != 2 {
                                anyhow::bail!("Expected 2 items for the sub-content of a message with an image.");
                            }
                            if !image_message.contains_key("type") {
                                anyhow::bail!("Expected `type` key in input message.");
                            }
                            if image_message["type"].is_right() {
                                anyhow::bail!("Expected string value in `type`.");
                            }
                            items.push(image_message["type"].as_ref().unwrap_left().clone())
                        }

                        fn get_content_and_url(
                            text_idx: usize,
                            url_idx: usize,
                            image_messages: &[HashMap<String, MessageInnerContent>],
                        ) -> Result<(String, String)> {
                            if image_messages[text_idx]["text"].is_right() {
                                anyhow::bail!("Expected string value in `text`.");
                            }
                            let content = image_messages[text_idx]["text"]
                                .as_ref()
                                .unwrap_left()
                                .clone();
                            if image_messages[url_idx]["image_url"].is_left()
                                || !image_messages[url_idx]["image_url"]
                                    .as_ref()
                                    .unwrap_right()
                                    .contains_key("url")
                            {
                                anyhow::bail!("Expected content of format {{`type`: `text`, `text`: ...}} and {{`type`: `url`, `image_url`: {{`url`: ...}}}}")
                            }
                            let url = image_messages[url_idx]["image_url"].as_ref().unwrap_right()
                                ["url"]
                                .clone();
                            Ok((content, url))
                        }
                        let mut message_map: IndexMap<
                            String,
                            Either<String, Vec<IndexMap<String, String>>>,
                        > = IndexMap::new();
                        message_map.insert("role".to_string(), Either::Left(message.role));
                        let (content, url) = if items[0] == "text" {
                            get_content_and_url(0, 1, &image_messages)?
                        } else {
                            get_content_and_url(1, 0, &image_messages)?
                        };

                        let mut content_map = Vec::new();
                        let mut content_image_map = IndexMap::new();
                        content_image_map.insert("type".to_string(), "image".to_string());
                        content_map.push(content_image_map);
                        let mut content_text_map = IndexMap::new();
                        content_text_map.insert("type".to_string(), "text".to_string());
                        content_text_map.insert("text".to_string(), content);
                        content_map.push(content_text_map);

                        message_map.insert("content".to_string(), Either::Right(content_map));
                        messages.push(message_map);
                        image_urls.push(url);
                    }
                }
            }
            if !image_urls.is_empty() {
                let mut images = Vec::new();
                for url_unparsed in image_urls {
                    let image = util::parse_image_url(&url_unparsed)
                        .await
                        .with_context(|| {
                            format!("Failed to parse image resource: {}", url_unparsed)
                        })?;

                    images.push(image);
                }
                RequestMessage::VisionChat { messages, images }
            } else {
                RequestMessage::Chat(messages)
            }
        }
        Either::Right(prompt) => {
            let mut messages = Vec::new();
            let mut message_map: IndexMap<String, Either<String, Vec<IndexMap<String, String>>>> =
                IndexMap::new();
            message_map.insert("role".to_string(), Either::Left("user".to_string()));
            message_map.insert("content".to_string(), Either::Left(prompt));
            messages.push(message_map);
            RequestMessage::Chat(messages)
        }
    };

    let dry_params = if let Some(dry_multiplier) = oairequest.dry_multiplier {
        Some(DrySamplingParams::new_with_defaults(
            dry_multiplier,
            oairequest.dry_sequence_breakers,
            oairequest.dry_base,
            oairequest.dry_allowed_length,
        )?)
    } else {
        None
    };

    let is_streaming = oairequest.stream.unwrap_or(false);
    Ok((
        Request::Normal(NormalRequest {
            id: state.next_request_id(),
            messages,
            sampling_params: SamplingParams {
                temperature: oairequest.temperature,
                top_k: oairequest.top_k,
                top_p: oairequest.top_p,
                min_p: oairequest.min_p,
                top_n_logprobs: oairequest.top_logprobs.unwrap_or(1),
                frequency_penalty: oairequest.frequency_penalty,
                presence_penalty: oairequest.presence_penalty,
                max_len: oairequest.max_tokens,
                stop_toks,
                logits_bias: oairequest.logit_bias,
                n_choices: oairequest.n_choices,
                dry_params,
            },
            response: tx,
            return_logprobs: oairequest.logprobs,
            is_streaming,
            suffix: None,
            constraint: match oairequest.grammar {
                Some(Grammar::Yacc(yacc)) => Constraint::Yacc(yacc),
                Some(Grammar::Regex(regex)) => Constraint::Regex(regex),
                None => Constraint::None,
            },
            adapters: oairequest.adapters,
            tool_choice: oairequest.tool_choice,
            tools: oairequest.tools,
            logits_processors: None,
        }),
        is_streaming,
    ))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Grammar {
    #[serde(rename = "regex")]
    Regex(String),
    #[serde(rename = "yacc")]
    Yacc(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    #[serde(with = "either::serde_untagged")]
    pub messages: Either<Vec<Message>, String>,
    #[serde(default = "default_model")]
    pub model: String,
    pub logit_bias: Option<HashMap<u32, f32>>,
    #[serde(default = "default_false")]
    pub logprobs: bool,
    pub top_logprobs: Option<usize>,
    pub max_tokens: Option<usize>,
    #[serde(rename = "n")]
    #[serde(default = "default_1usize")]
    pub n_choices: usize,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    #[serde(rename = "stop")]
    pub stop_seqs: Option<StopTokens>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,

    // mistral.rs additional
    pub top_k: Option<usize>,
    pub grammar: Option<Grammar>,
    pub adapters: Option<Vec<String>>,
    pub min_p: Option<f64>,
    pub dry_multiplier: Option<f32>,
    pub dry_base: Option<f32>,
    pub dry_allowed_length: Option<usize>,
    pub dry_sequence_breakers: Option<Vec<String>>,
}

fn default_false() -> bool {
    false
}

fn default_1usize() -> usize {
    1
}

fn default_model() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub content: MessageContent,
    pub role: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageContent(
    #[serde(with = "either::serde_untagged")]
    Either<String, Vec<HashMap<String, MessageInnerContent>>>,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageInnerContent(
    #[serde(with = "either::serde_untagged")] Either<String, HashMap<String, String>>,
);

impl Deref for MessageInnerContent {
    type Target = Either<String, HashMap<String, String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StopTokens {
    Multi(Vec<String>),
    Single(String),
}

pub mod util {
    use image::DynamicImage;
    use tokio::{
        fs::{self, File},
        io::AsyncReadExt,
    };

    pub async fn parse_image_url(url_unparsed: &str) -> Result<DynamicImage, anyhow::Error> {
        let url = if let Ok(url) = url::Url::parse(url_unparsed) {
            url
        } else if File::open(url_unparsed).await.is_ok() {
            url::Url::from_file_path(std::path::absolute(url_unparsed)?)
                .map_err(|_| anyhow::anyhow!("Could not parse file path: {}", url_unparsed))?
        } else {
            url::Url::parse(&format!("data:image/png;base64,{}", url_unparsed))
                .map_err(|_| anyhow::anyhow!("Could not parse as base64 data: {}", url_unparsed))?
        };

        let bytes = if url.scheme() == "http" || url.scheme() == "https" {
            // Read from http
            match reqwest::get(url.clone()).await {
                Ok(http_resp) => http_resp.bytes().await?.to_vec(),
                Err(e) => anyhow::bail!(e),
            }
        } else if url.scheme() == "file" {
            let path = url
                .to_file_path()
                .map_err(|_| anyhow::anyhow!("Could not parse file path: {}", url))?;

            if let Ok(mut f) = File::open(&path).await {
                // Read from local file
                let metadata = fs::metadata(&path).await?;
                let mut buffer = vec![0; metadata.len() as usize];
                f.read_exact(&mut buffer).await?;
                buffer
            } else {
                anyhow::bail!("Could not open file at path: {}", url);
            }
        } else if url.scheme() == "data" {
            // Decode with base64
            let data_url = data_url::DataUrl::process(url.as_str())?;
            data_url.decode_to_vec()?.0
        } else {
            anyhow::bail!("Unsupported URL scheme: {}", url.scheme());
        };

        Ok(image::load_from_memory(&bytes)?)
    }
}

pub enum ChatCompletionResponder {
    Sse(Sse<Streamer>),
    Json(ChatCompletionResponse),
    ModelError(String, ChatCompletionResponse),
    InternalError(Box<dyn Error>),
    ValidationError(Box<dyn Error>),
}

impl IntoResponse for ChatCompletionResponder {
    fn into_response(self) -> axum::response::Response {
        match self {
            ChatCompletionResponder::Sse(s) => s.into_response(),
            ChatCompletionResponder::Json(s) => Json(s).into_response(),
            ChatCompletionResponder::InternalError(e) => {
                JsonError::new(e.to_string()).to_response(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
            ChatCompletionResponder::ValidationError(e) => {
                JsonError::new(e.to_string()).to_response(http::StatusCode::UNPROCESSABLE_ENTITY)
            }
            ChatCompletionResponder::ModelError(msg, response) => {
                JsonModelError::new(msg, response)
                    .to_response(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

trait ErrorToResponse: Serialize {
    fn to_response(&self, code: StatusCode) -> axum::response::Response {
        let mut r = Json(self).into_response();
        *r.status_mut() = code;
        r
    }
}

#[derive(Serialize)]
struct JsonError {
    message: String,
}

impl JsonError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ErrorToResponse for JsonError {}

#[derive(Serialize)]
struct JsonModelError {
    message: String,
    partial_response: ChatCompletionResponse,
}

impl JsonModelError {
    fn new(message: String, partial_response: ChatCompletionResponse) -> Self {
        Self {
            message,
            partial_response,
        }
    }
}

impl ErrorToResponse for JsonModelError {}

pub struct Streamer {
    rx: Receiver<Response>,
    is_done: bool,
    state: Arc<MistralRs>,
}

impl futures::Stream for Streamer {
    type Item = Result<Event, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.is_done {
            return Poll::Ready(None);
        }
        match self.rx.try_recv() {
            Ok(resp) => match resp {
                Response::ModelError(msg, _) => {
                    MistralRs::maybe_log_error(
                        self.state.clone(),
                        &ModelErrorMessage(msg.to_string()),
                    );
                    Poll::Ready(Some(Ok(Event::default().data(msg))))
                }
                Response::ValidationError(e) => {
                    Poll::Ready(Some(Ok(Event::default().data(e.to_string()))))
                }
                Response::InternalError(e) => {
                    MistralRs::maybe_log_error(self.state.clone(), &*e);
                    Poll::Ready(Some(Ok(Event::default().data(e.to_string()))))
                }
                Response::Chunk(response) => {
                    if response.choices.iter().all(|x| x.finish_reason.is_some()) {
                        self.is_done = true;
                    }
                    MistralRs::maybe_log_response(self.state.clone(), &response);
                    Poll::Ready(Some(Event::default().json_data(response)))
                }
                Response::Done(_) => unreachable!(),
                Response::CompletionDone(_) => unreachable!(),
                Response::CompletionModelError(_, _) => unreachable!(),
                Response::CompletionChunk(_) => unreachable!(),
            },
            Err(_) => Poll::Pending,
        }
    }
}

#[derive(Debug)]
struct ModelErrorMessage(String);
impl std::fmt::Display for ModelErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ModelErrorMessage {}
