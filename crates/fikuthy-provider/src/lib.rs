use anyhow::{Context, Result};
use reqwest::{blocking::Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader},
    thread,
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    OpenRouter,
    OpenAi,
    Groq,
    Together,
    DeepSeek,
    Fireworks,
    Nvidia,
    Ollama,
    Custom,
}

impl ProviderKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openrouter" => Ok(Self::OpenRouter),
            "openai" => Ok(Self::OpenAi),
            "groq" => Ok(Self::Groq),
            "together" => Ok(Self::Together),
            "deepseek" => Ok(Self::DeepSeek),
            "fireworks" => Ok(Self::Fireworks),
            "nvidia" | "nvidia-nim" | "nim" => Ok(Self::Nvidia),
            "ollama" | "local" => Ok(Self::Ollama),
            "custom" | "openai-compatible" => Ok(Self::Custom),
            other => anyhow::bail!("unsupported provider '{other}'"),
        }
    }
    pub fn id(&self) -> &'static str {
        match self {
            Self::OpenRouter => "openrouter",
            Self::OpenAi => "openai",
            Self::Groq => "groq",
            Self::Together => "together",
            Self::DeepSeek => "deepseek",
            Self::Fireworks => "fireworks",
            Self::Nvidia => "nvidia",
            Self::Ollama => "ollama",
            Self::Custom => "custom",
        }
    }
    fn defaults(&self) -> (&'static str, &'static str, Option<&'static str>) {
        match self {
            Self::OpenRouter => (
                "https://openrouter.ai/api/v1",
                "openrouter/free",
                Some("OPENROUTER_API_KEY"),
            ),
            Self::OpenAi => (
                "https://api.openai.com/v1",
                "gpt-4o-mini",
                Some("OPENAI_API_KEY"),
            ),
            Self::Groq => (
                "https://api.groq.com/openai/v1",
                "llama-3.3-70b-versatile",
                Some("GROQ_API_KEY"),
            ),
            Self::Together => (
                "https://api.together.xyz/v1",
                "meta-llama/Llama-3.3-70B-Instruct-Turbo",
                Some("TOGETHER_API_KEY"),
            ),
            Self::DeepSeek => (
                "https://api.deepseek.com/v1",
                "deepseek-chat",
                Some("DEEPSEEK_API_KEY"),
            ),
            Self::Fireworks => (
                "https://api.fireworks.ai/inference/v1",
                "accounts/fireworks/models/llama-v3p3-70b-instruct",
                Some("FIREWORKS_API_KEY"),
            ),
            Self::Nvidia => (
                "https://integrate.api.nvidia.com/v1",
                "nvidia/nemotron-3-super-120b-a12b",
                Some("NVIDIA_API_KEY"),
            ),
            Self::Ollama => ("http://127.0.0.1:11434/v1", "qwen2.5-coder:7b", None),
            Self::Custom => (
                "http://127.0.0.1:8000/v1",
                "default",
                Some("FIKUTHY_API_KEY"),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Gateway {
    client: Client,
    kind: ProviderKind,
    api_key: Option<String>,
    base_url: String,
    model: String,
    max_retries: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStatus {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub credential_source: Option<String>,
    pub configured: bool,
}
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    error: Option<ApiError>,
}
#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}
#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}
#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelRecord>,
}
#[derive(Debug, Deserialize)]
struct ModelRecord {
    id: String,
}
#[derive(Debug, Deserialize)]
struct StreamPayload {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    error: Option<ApiError>,
}
#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: StreamDelta,
}
#[derive(Debug, Default, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

impl Gateway {
    pub fn from_environment() -> Result<Self> {
        let provider = if let Ok(value) = std::env::var("FIKUTHY_PROVIDER") {
            ProviderKind::parse(&value)?
        } else if environment_has_value("OPENROUTER_API_KEY") {
            ProviderKind::OpenRouter
        } else if environment_has_value("OPENAI_API_KEY") {
            ProviderKind::OpenAi
        } else if environment_has_value("GROQ_API_KEY") {
            ProviderKind::Groq
        } else if environment_has_value("TOGETHER_API_KEY") {
            ProviderKind::Together
        } else if environment_has_value("DEEPSEEK_API_KEY") {
            ProviderKind::DeepSeek
        } else if environment_has_value("FIREWORKS_API_KEY") {
            ProviderKind::Fireworks
        } else if environment_has_value("NVIDIA_API_KEY") {
            ProviderKind::Nvidia
        } else {
            anyhow::bail!("no AI gateway is configured; set FIKUTHY_PROVIDER or a supported provider API key")
        };
        Self::new_from_environment(provider)
    }
    pub fn new_from_environment(kind: ProviderKind) -> Result<Self> {
        let (default_url, default_model, key_variable) = kind.defaults();
        let base_url =
            std::env::var("FIKUTHY_PROVIDER_URL").unwrap_or_else(|_| default_url.into());
        validate_base_url(&base_url)?;
        let source = std::env::var("FIKUTHY_API_KEY")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|_| "FIKUTHY_API_KEY")
            .or_else(|| {
                key_variable.and_then(|name| {
                    std::env::var(name)
                        .ok()
                        .filter(|v| !v.trim().is_empty())
                        .map(|_| name)
                })
            });
        let api_key = source.and_then(|name| std::env::var(name).ok());
        if kind != ProviderKind::Ollama && api_key.is_none() {
            anyhow::bail!(
                "{} is not configured; set {} or FIKUTHY_API_KEY",
                kind.id(),
                key_variable.unwrap_or("FIKUTHY_API_KEY")
            );
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .build()
            .context("failed to build provider HTTP client")?;
        Ok(Self {
            client,
            kind,
            api_key,
            base_url: base_url.trim_end_matches('/').into(),
            model: std::env::var("FIKUTHY_MODEL").unwrap_or_else(|_| default_model.into()),
            max_retries: 2,
        })
    }
    pub fn status_from_environment(kind: ProviderKind) -> ProviderStatus {
        let (default_url, default_model, key_variable) = kind.defaults();
        let source = if std::env::var("FIKUTHY_API_KEY").is_ok_and(|v| !v.trim().is_empty()) {
            Some("FIKUTHY_API_KEY".into())
        } else {
            key_variable
                .filter(|name| std::env::var(name).is_ok_and(|v| !v.trim().is_empty()))
                .map(str::to_string)
        };
        ProviderStatus {
            provider: kind.id().into(),
            model: std::env::var("FIKUTHY_MODEL").unwrap_or_else(|_| default_model.into()),
            base_url: std::env::var("FIKUTHY_PROVIDER_URL")
                .unwrap_or_else(|_| default_url.into()),
            configured: kind == ProviderKind::Ollama || source.is_some(),
            credential_source: source,
        }
    }
    pub fn provider(&self) -> &str {
        self.kind.id()
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    /// Model ID for display, stripping the provider prefix when it already
    /// appears in the model ID (e.g. nvidia/nemotron → nemotron for the nvidia provider).
    pub fn display_model(&self) -> &str {
        let prefix = format!("{}/", self.kind.id());
        self.model.strip_prefix(&prefix).unwrap_or(&self.model)
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn health_check(&self) -> Result<StatusCode> {
        let response = self
            .authorize(self.client.get(format!("{}/models", self.base_url)))
            .send()
            .context("provider health request failed")?;
        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("{} health check failed with HTTP {status}", self.kind.id());
        }
        Ok(status)
    }
    pub fn models(&self) -> Result<Vec<String>> {
        let response = self
            .authorize(self.client.get(format!("{}/models", self.base_url)))
            .send()
            .context("provider model-list request failed")?;
        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("{} model list failed with HTTP {status}", self.kind.id());
        }
        let payload: ModelsResponse = response
            .json()
            .context("provider returned an invalid model list")?;
        let mut models = payload
            .data
            .into_iter()
            .map(|model| model.id)
            .collect::<Vec<_>>();
        models.sort();
        models.dedup();
        Ok(models)
    }
    pub fn validate_model(&self) -> Result<()> {
        let models = self.models()?;
        if models.is_empty() || models.iter().any(|model| model == &self.model) {
            return Ok(());
        }
        anyhow::bail!(
            "model '{}' is not exposed by {}; choose one returned by `fikuthy models list`",
            self.model,
            self.kind.id()
        )
    }
    pub fn complete(&self, messages: &[ChatMessage], temperature: f32) -> Result<String> {
        let body = serde_json::json!({"model": self.model, "messages": messages, "temperature": temperature, "max_tokens": 1200});
        for attempt in 0..=self.max_retries {
            let response = self
                .authorize(
                    self.client
                        .post(format!("{}/chat/completions", self.base_url)),
                )
                .json(&body)
                .send();
            let response = match response {
                Ok(v) => v,
                Err(error)
                    if attempt < self.max_retries && (error.is_timeout() || error.is_connect()) =>
                {
                    thread::sleep(retry_delay(attempt));
                    continue;
                }
                Err(error) => return Err(error).context("provider request failed"),
            };
            let status = response.status();
            let retryable = status.as_u16() == 429 || status.is_server_error();
            let raw = response
                .text()
                .context("failed to read provider response")?;
            if retryable && attempt < self.max_retries {
                thread::sleep(retry_delay(attempt));
                continue;
            }
            let payload: ChatResponse = serde_json::from_str(&raw)
                .with_context(|| format!("provider returned invalid JSON (HTTP {status})"))?;
            if !status.is_success() {
                anyhow::bail!(
                    "provider request rejected: {}",
                    payload
                        .error
                        .map(|e| e.message)
                        .unwrap_or_else(|| format!("HTTP {status}"))
                );
            }
            return payload
                .choices
                .into_iter()
                .next()
                .map(|c| c.message.content)
                .filter(|v| !v.trim().is_empty())
                .context("provider returned no assistant content");
        }
        unreachable!("retry loop always returns")
    }
    pub fn complete_streaming<F>(&self, messages: &[ChatMessage], mut on_delta: F) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        let body = serde_json::json!({"model": self.model, "messages": messages, "temperature": 0.2, "max_tokens": 1600, "stream": true});
        for attempt in 0..=self.max_retries {
            let response = self
                .authorize(
                    self.client
                        .post(format!("{}/chat/completions", self.base_url)),
                )
                .json(&body)
                .send();
            let response = match response {
                Ok(v) => v,
                Err(error)
                    if attempt < self.max_retries && (error.is_timeout() || error.is_connect()) =>
                {
                    thread::sleep(retry_delay(attempt));
                    continue;
                }
                Err(error) => return Err(error).context("streaming provider request failed"),
            };
            let status = response.status();
            let retryable = status.as_u16() == 429 || status.is_server_error();
            if retryable && attempt < self.max_retries {
                thread::sleep(retry_delay(attempt));
                continue;
            }
            if !status.is_success() {
                anyhow::bail!("streaming provider request rejected (HTTP {status})");
            }
            let mut complete = String::new();
            for line in BufReader::new(response).lines() {
                let line = line.context("failed reading provider stream")?;
                let Some(data) = line.strip_prefix("data:") else {
                    continue;
                };
                let data = data.trim();
                if data == "[DONE]" {
                    break;
                }
                let payload: StreamPayload =
                    serde_json::from_str(data).context("provider stream contained invalid JSON")?;
                if let Some(error) = payload.error {
                    anyhow::bail!("provider stream failed: {}", error.message);
                }
                for choice in payload.choices {
                    if let Some(delta) = choice.delta.content.filter(|v| !v.is_empty()) {
                        on_delta(&delta)?;
                        complete.push_str(&delta);
                    }
                }
            }
            if complete.trim().is_empty() {
                anyhow::bail!("provider stream completed without assistant content");
            }
            return Ok(complete);
        }
        unreachable!("retry loop always returns")
    }
    pub fn complete_json<T: for<'de> Deserialize<'de>>(
        &self,
        messages: &[ChatMessage],
    ) -> Result<T> {
        let raw = self.complete(messages, 0.1)?;
        let cleaned = raw
            .trim()
            .strip_prefix("```json")
            .unwrap_or(raw.trim())
            .strip_suffix("```")
            .unwrap_or_else(|| raw.trim())
            .trim();
        serde_json::from_str(cleaned).context("provider response was not valid requested JSON")
    }
    fn authorize(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        let request = if let Some(key) = &self.api_key {
            request.bearer_auth(key)
        } else {
            request
        };
        if self.kind == ProviderKind::OpenRouter {
            request
                .header("HTTP-Referer", "https://github.com/fikuthy/fikuthy")
                .header("X-Title", "Fikuthy Agent Terminal")
        } else {
            request
        }
    }
}

pub type OpenRouter = Gateway;
pub fn supported_providers() -> Vec<ProviderStatus> {
    [
        ProviderKind::OpenRouter,
        ProviderKind::OpenAi,
        ProviderKind::Groq,
        ProviderKind::Together,
        ProviderKind::DeepSeek,
        ProviderKind::Fireworks,
        ProviderKind::Nvidia,
        ProviderKind::Ollama,
        ProviderKind::Custom,
    ]
    .into_iter()
    .map(Gateway::status_from_environment)
    .collect()
}

pub fn has_provider_configuration() -> bool {
    std::env::var("FIKUTHY_PROVIDER").is_ok_and(|value| !value.trim().is_empty())
        || [
            "FIKUTHY_API_KEY",
            "OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "GROQ_API_KEY",
            "TOGETHER_API_KEY",
            "DEEPSEEK_API_KEY",
            "FIREWORKS_API_KEY",
            "NVIDIA_API_KEY",
        ]
        .into_iter()
        .any(environment_has_value)
}

fn environment_has_value(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| !value.trim().is_empty())
}
fn validate_base_url(value: &str) -> Result<()> {
    let url = Url::parse(value).context("FIKUTHY_PROVIDER_URL is invalid")?;
    let local_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !local_http {
        anyhow::bail!("provider URL must use HTTPS; HTTP is allowed only for loopback hosts");
    }
    Ok(())
}
fn retry_delay(attempt: usize) -> Duration {
    Duration::from_millis(250 * (1_u64 << attempt.min(3)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    #[test]
    fn chat_messages_serialize_for_openai_compatible_api() {
        let value = serde_json::to_value(ChatMessage {
            role: "user".into(),
            content: "hello".into(),
        })
        .unwrap();
        assert_eq!(value["role"], "user");
    }
    #[test]
    fn provider_names_are_explicit() {
        assert_eq!(ProviderKind::parse("ollama").unwrap(), ProviderKind::Ollama);
        assert_eq!(ProviderKind::parse("nim").unwrap(), ProviderKind::Nvidia);
        assert!(ProviderKind::parse("unknown").is_err());
    }
    #[test]
    fn retry_delay_is_bounded_exponential_backoff() {
        assert_eq!(retry_delay(0), Duration::from_millis(250));
        assert_eq!(retry_delay(20), Duration::from_millis(2000));
    }
    #[test]
    fn rejects_non_loopback_plain_http() {
        assert!(validate_base_url("http://example.com/v1").is_err());
        assert!(validate_base_url("http://127.0.0.1:8000/v1").is_ok());
    }
    #[test]
    fn streams_openai_compatible_deltas_in_order() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0_u8; 8192];
            let read = socket.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..read]).contains("POST /v1/chat/completions"));
            let body = "data: {\"choices\":[{\"delta\":{\"content\":\"real \"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"time\"}}]}\n\ndata: [DONE]\n\n";
            write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        });
        let gateway = Gateway {
            client: Client::builder().build().unwrap(),
            kind: ProviderKind::Custom,
            api_key: Some("test-only".into()),
            base_url: format!("http://{address}/v1"),
            model: "fixture".into(),
            max_retries: 0,
        };
        let mut deltas = Vec::new();
        let complete = gateway
            .complete_streaming(
                &[ChatMessage {
                    role: "user".into(),
                    content: "hello".into(),
                }],
                |delta| {
                    deltas.push(delta.to_string());
                    Ok(())
                },
            )
            .unwrap();
        server.join().unwrap();
        assert_eq!(deltas, ["real ", "time"]);
        assert_eq!(complete, "real time");
    }
}
