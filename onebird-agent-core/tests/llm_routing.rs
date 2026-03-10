use onebird_agent_core::llm::{LlmClient, LlmError, RigLlmClient, RigLlmConfig, RigProviderConfig};
use onebird_agent_core::types::{
    AgentContext, AgentLoopConfig, AgentMessage, LlmModelId, MessageContent, Role, ThinkingLevel,
};

fn empty_context() -> AgentContext {
    AgentContext {
        system_prompt: String::new(),
        messages: Vec::new(),
        tools: Vec::new(),
    }
}

fn dummy_config(provider: &str, model: &str) -> (AgentContext, AgentLoopConfig, RigLlmClient) {
    let ctx = empty_context();
    let loop_cfg = AgentLoopConfig {
        model: LlmModelId {
            provider: provider.to_string(),
            model: model.to_string(),
        },
        thinking_level: ThinkingLevel::Off,
    };
    let rig_cfg = RigLlmConfig {
        model: loop_cfg.model.clone(),
        provider: RigProviderConfig {
            base_url: None,
            api_key: None,
        },
    };
    (ctx, loop_cfg, RigLlmClient::new(rig_cfg))
}

#[test]
fn unknown_provider_returns_error() {
    let (ctx, loop_cfg, client) = dummy_config("non-existent-provider", "fake-model");

    let result = client.complete(&ctx, &loop_cfg);
    assert!(result.is_err());
    if let Err(LlmError::Backend(msg)) = result {
        assert!(
            msg.contains("unknown provider"),
            "unexpected error message: {msg}"
        );
    }
}

#[test]
fn eternalai_provider_is_explicitly_unimplemented() {
    let (ctx, loop_cfg, client) = dummy_config("eternalai", "fake-model");

    let result = client.complete(&ctx, &loop_cfg);
    assert!(result.is_err());
    if let Err(LlmError::Backend(msg)) = result {
        assert!(
            msg.contains("EternalAI provider not yet wired"),
            "unexpected error message: {msg}"
        );
    }
}

struct FakeLlm;

impl LlmClient for FakeLlm {
    fn complete(
        &self,
        _context: &AgentContext,
        _config: &AgentLoopConfig,
    ) -> Result<String, LlmError> {
        Ok("fake-response".to_string())
    }
}

#[test]
fn stream_assistant_response_wraps_text_into_message() {
    use onebird_agent_core::llm::stream_assistant_response;

    let mut ctx = AgentContext {
        system_prompt: "test".to_string(),
        messages: vec![AgentMessage {
            role: Role::User,
            content: vec![MessageContent::Text {
                text: "hi".to_string(),
            }],
            timestamp: None,
            metadata: Default::default(),
        }],
        tools: Vec::new(),
    };

    let loop_cfg = AgentLoopConfig {
        model: LlmModelId {
            provider: "test".to_string(),
            model: "fake".to_string(),
        },
        thinking_level: ThinkingLevel::Off,
    };

    let client = FakeLlm;

    let msg =
        stream_assistant_response(&client, &mut ctx, &loop_cfg).expect("stream should succeed");

    assert!(matches!(msg.role, Role::Assistant));
    assert!(matches!(
        msg.content.first(),
        Some(MessageContent::Text { text }) if text == "fake-response"
    ));
    assert!(matches!(
        ctx.messages.last(),
        Some(AgentMessage {
            role: Role::Assistant,
            ..
        })
    ));
}

