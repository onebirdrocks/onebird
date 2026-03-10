//! Stateless agent loop and tool execution.
//!
//! 第一阶段：实现最小可用的无状态对话 loop，不处理工具调用 / steering / follow-up。

use crate::llm::{stream_assistant_response, LlmClient, LlmError};
use crate::types::{
    AgentContext, AgentContextSnapshot, AgentEvent, AgentLoopConfig, AgentMessage, Role,
};
use uuid::Uuid;

/// Run the agent loop starting from new user messages.
pub fn agent_loop(
    prompts: Vec<AgentMessage>,
    mut context: AgentContext,
    config: AgentLoopConfig,
    client: &dyn LlmClient,
    on_event: &mut dyn FnMut(AgentEvent),
) -> Result<Vec<AgentMessage>, LlmError> {
    // 初始事件
    on_event(AgentEvent::AgentStart {
        context: AgentContextSnapshot {
            system_prompt: context.system_prompt.clone(),
            messages: context.messages.clone(),
        },
    });

    let turn_id = Uuid::new_v4();
    on_event(AgentEvent::TurnStart { turn_id });

    let mut new_messages = Vec::new();

    // 写入用户 prompt
    for msg in prompts {
        if matches!(msg.role, Role::User) {
            on_event(AgentEvent::MessageStart {
                message: msg.clone(),
            });
            on_event(AgentEvent::MessageEnd {
                message: msg.clone(),
            });
            context.messages.push(msg.clone());
            new_messages.push(msg.clone());
        }
    }

    // 调用一次 LLM
    let assistant = stream_assistant_response(client, &mut context, &config)?;
    on_event(AgentEvent::MessageStart {
        message: assistant.clone(),
    });
    on_event(AgentEvent::MessageEnd {
        message: assistant.clone(),
    });
    new_messages.push(assistant);

    on_event(AgentEvent::TurnEnd {
        turn_id,
        tool_results: Vec::new(),
    });

    on_event(AgentEvent::AgentEnd {
        messages: context.messages.clone(),
    });

    Ok(new_messages)
}

/// Continue the agent loop from an existing context.
pub fn agent_loop_continue(
    context: AgentContext,
    config: AgentLoopConfig,
    client: &dyn LlmClient,
    on_event: &mut dyn FnMut(AgentEvent),
) -> Result<Vec<AgentMessage>, LlmError> {
    // 这里直接复用 agent_loop，且不追加新的 user 消息。
    agent_loop(Vec::new(), context, config, client, on_event)
}

