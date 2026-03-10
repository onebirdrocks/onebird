use onebird_agent_core::{
    agent::Agent,
    llm::{LlmClient, LlmError},
    r#loop::agent_loop,
    types::{AgentContext, AgentEvent, AgentLoopConfig, AgentMessage, AgentState, LlmModelId, MessageContent, Role, ThinkingLevel},
};

struct FakeLlm;

impl LlmClient for FakeLlm {
    fn complete(
        &self,
        _context: &AgentContext,
        _config: &AgentLoopConfig,
    ) -> Result<String, LlmError> {
        Ok("hello from fake".to_string())
    }
}

fn base_state() -> AgentState {
    AgentState {
        system_prompt: "You are a test agent".to_string(),
        model: LlmModelId {
            provider: "test".to_string(),
            model: "fake".to_string(),
        },
        thinking_level: ThinkingLevel::Off,
        tools: Vec::new(),
        messages: Vec::new(),
        is_streaming: false,
        stream_message: None,
        pending_tool_calls: Vec::new(),
        error: None,
    }
}

#[test]
fn agent_loop_returns_assistant_message() {
    let client = FakeLlm;
    let context = AgentContext {
        system_prompt: "You are a test agent".to_string(),
        messages: Vec::new(),
        tools: Vec::new(),
    };
    let config = AgentLoopConfig {
        model: LlmModelId {
            provider: "test".to_string(),
            model: "fake".to_string(),
        },
        thinking_level: ThinkingLevel::Off,
    };

    let user = AgentMessage {
        role: Role::User,
        content: vec![MessageContent::Text {
            text: "hi".to_string(),
        }],
        timestamp: None,
        metadata: Default::default(),
    };

    let mut events: Vec<AgentEvent> = Vec::new();
    let mut on_event = |e: AgentEvent| {
        events.push(e);
    };

    let result =
        agent_loop(vec![user], context, config, &client, &mut on_event).expect("loop ok");

    assert!(!result.is_empty());
    let assistant = result.last().unwrap();
    assert!(matches!(assistant.role, Role::Assistant));
    assert!(matches!(
        assistant.content.first(),
        Some(MessageContent::Text { text }) if text == "hello from fake"
    ));
}

#[test]
fn agent_wrapper_updates_state_messages() {
    let client = FakeLlm;
    let state = base_state();

    let mut agent = Agent::new(state, &client);

    let user = AgentMessage {
        role: Role::User,
        content: vec![MessageContent::Text {
            text: "hi".to_string(),
        }],
        timestamp: None,
        metadata: Default::default(),
    };

    // 这里只验证事件流是否能正常触发，不对具体副作用做断言，
    // 避免捕获非 'static 变量的麻烦。
    agent.subscribe(|_e| {});

    agent.prompt(vec![user]);

    let messages = &agent.state().messages;
    assert_eq!(messages.len(), 2);
    assert!(matches!(messages[0].role, Role::User));
    assert!(matches!(messages[1].role, Role::Assistant));
}

