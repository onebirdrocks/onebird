//! Stateful Agent wrapper around the stateless loop.
//!
//! 第一阶段：提供一个最小的有状态 `Agent`，封装对 `agent_loop` 的调用，
//! 支持订阅事件，但暂不处理 steering / follow-up / abort 等高级功能。

use crate::llm::LlmClient;
use crate::r#loop::agent_loop;
use crate::types::{AgentContext, AgentEvent, AgentLoopConfig, AgentMessage, AgentState};

pub struct Agent<'a> {
    state: AgentState,
    llm_client: &'a dyn LlmClient,
    listeners: Vec<Box<dyn Fn(&AgentEvent) + Send + Sync>>,
}

impl<'a> Agent<'a> {
    pub fn new(state: AgentState, llm_client: &'a dyn LlmClient) -> Self {
        Self {
            state,
            llm_client,
            listeners: Vec::new(),
        }
    }

    pub fn state(&self) -> &AgentState {
        &self.state
    }

    pub fn subscribe<F>(&mut self, f: F)
    where
        F: Fn(&AgentEvent) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    fn emit(&self, event: AgentEvent) {
        for l in &self.listeners {
            l(&event);
        }
    }

    /// 发送一条或多条用户消息并运行一轮对话。
    pub fn prompt(&mut self, prompts: Vec<AgentMessage>) {
        let context = AgentContext {
            system_prompt: self.state.system_prompt.clone(),
            messages: self.state.messages.clone(),
            tools: Vec::new(),
        };

        let config = AgentLoopConfig {
            model: self.state.model.clone(),
            thinking_level: self.state.thinking_level,
        };

        let mut on_event = |e: AgentEvent| {
            self.emit(e);
        };

        if let Ok(new_messages) =
            agent_loop(prompts, context, config, self.llm_client, &mut on_event)
        {
            self.state.messages.extend(new_messages);
        }
    }
}

