//! LLM integration based on Rig & provider routing.
//!
//! 提供 `LlmClient` 抽象和 `RigLlmClient`，支持 Rig 内置的多家 provider：
//! OpenAI、Anthropic、Gemini、Cohere、Perplexity、xAI、DeepSeek、Azure OpenAI、Mira 等。

use crate::types::{
    AgentContext, AgentLoopConfig, AgentMessage, LlmModelId, MessageContent, Role,
};
use thiserror::Error;

/// Error type for LLM operations.
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("LLM backend error: {0}")]
    Backend(String),
}

/// 简化版 LLM 抽象：给定上下文，返回一条新的 assistant 文本。
pub trait LlmClient: Send + Sync {
    fn complete(
        &self,
        context: &AgentContext,
        config: &AgentLoopConfig,
    ) -> Result<String, LlmError>;
}

/// 单个 provider 的配置（覆盖 endpoint / api_key 等）。
#[derive(Debug, Clone)]
pub struct RigProviderConfig {
    /// 可选自定义 base_url（暂未在所有 provider 上用到，预留字段）。
    pub base_url: Option<String>,
    /// API key；若为 None，则由 Rig 自行从环境变量读取。
    pub api_key: Option<String>,
}

/// Rig 多 provider 的统一配置。
#[derive(Debug, Clone)]
pub struct RigLlmConfig {
    pub model: LlmModelId,
    pub provider: RigProviderConfig,
}

/// 基于 Rig 的多 provider LLM 客户端实现。
pub struct RigLlmClient {
    cfg: RigLlmConfig,
}

impl RigLlmClient {
    pub fn new(cfg: RigLlmConfig) -> Self {
        Self { cfg }
    }

    #[allow(dead_code)]
    fn build_prompt(&self, ctx: &AgentContext) -> (String, String) {
        let system = ctx.system_prompt.clone();
        let mut user_parts = Vec::new();

        for m in &ctx.messages {
            let prefix = match m.role {
                Role::System => "System",
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::ToolResult => "Tool",
            };
            let text = m
                .content
                .iter()
                .filter_map(|c| match c {
                    MessageContent::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !text.is_empty() {
                user_parts.push(format!("{prefix}: {text}"));
            }
        }

        let user_prompt = if user_parts.is_empty() {
            "User: (empty)".to_string()
        } else {
            user_parts.join("\n")
        };

        (system, user_prompt)
    }

    #[allow(dead_code)]
    fn completion_error<E: std::fmt::Display>(e: E) -> LlmError {
        LlmError::Backend(e.to_string())
    }

    // -------- 各 provider 的具体实现 --------

    fn complete_openai(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: openai provider not implemented yet".to_string(),
        ))
    }

    fn complete_anthropic(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: anthropic provider not implemented yet".to_string(),
        ))
    }

    fn complete_gemini(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: gemini provider not implemented yet".to_string(),
        ))
    }

    fn complete_cohere(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: cohere provider not implemented yet".to_string(),
        ))
    }

    fn complete_perplexity(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: perplexity provider not implemented yet".to_string(),
        ))
    }

    fn complete_xai(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: xai provider not implemented yet".to_string(),
        ))
    }

    fn complete_deepseek(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: deepseek provider not implemented yet".to_string(),
        ))
    }

    fn complete_azure(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: azure provider not implemented yet".to_string(),
        ))
    }

    fn complete_mira(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "RigLlmClient: mira provider not implemented yet".to_string(),
        ))
    }

    fn complete_eternalai(&self, _ctx: &AgentContext) -> Result<String, LlmError> {
        Err(LlmError::Backend(
            "EternalAI provider not yet wired in RigLlmClient".to_string(),
        ))
    }
}

impl LlmClient for RigLlmClient {
    fn complete(
        &self,
        context: &AgentContext,
        _config: &AgentLoopConfig,
    ) -> Result<String, LlmError> {
        match self.cfg.model.provider.as_str() {
            "openai" => self.complete_openai(context),
            "anthropic" => self.complete_anthropic(context),
            "gemini" | "google" => self.complete_gemini(context),
            "cohere" => self.complete_cohere(context),
            "perplexity" => self.complete_perplexity(context),
            "xai" => self.complete_xai(context),
            "deepseek" => self.complete_deepseek(context),
            "azure" | "azure-openai" => self.complete_azure(context),
            "mira" => self.complete_mira(context),
            "eternalai" => self.complete_eternalai(context),
            other => Err(LlmError::Backend(format!(
                "unknown provider '{}'",
                other
            ))),
        }
    }
}

/// 把 LLM 返回的文本包装成 `AgentMessage` 并追加到上下文中。
pub fn stream_assistant_response(
    client: &dyn LlmClient,
    context: &mut AgentContext,
    config: &AgentLoopConfig,
) -> Result<AgentMessage, LlmError> {
    let text = client.complete(context, config)?;

    let message = AgentMessage {
        role: Role::Assistant,
        content: vec![MessageContent::Text { text }],
        timestamp: None,
        metadata: Default::default(),
    };

    context.messages.push(message.clone());
    Ok(message)
}

