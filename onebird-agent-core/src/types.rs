use std::collections::HashMap;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Role of a message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
    ToolResult,
}

/// Content of a single message segment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { text: String },
    /// A tool call issued by the assistant.
    ToolCall {
        id: String,
        name: String,
        /// Raw JSON string with the tool arguments.
        arguments_json: String,
    },
}

/// A message in the agent conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentMessage {
    pub role: Role,
    pub content: Vec<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<SystemTime>,
    /// Arbitrary metadata for extensions and UI.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

/// Result returned by a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolResult<TDetails = Value> {
    /// Text content returned to the model / user.
    pub content: Vec<String>,
    /// Structured details for UI, logging, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<TDetails>,
    /// Whether the tool execution failed.
    pub is_error: bool,
}

/// Trait implemented by all tools callable from the agent loop.
pub trait AgentTool: Send + Sync {
    /// Structured details type returned by this tool.
    type Details: Send + Sync + 'static;

    /// Unique name of this tool (used in tool calls).
    fn name(&self) -> &str;

    /// Human readable label for UI.
    fn label(&self) -> &str;

    /// Execute the tool.
    ///
    /// `arguments_json` contains the raw JSON argument object from the model.
    /// Implementations are responsible for deserializing it as needed.
    fn execute(
        &self,
        tool_call_id: &str,
        arguments_json: &str,
    ) -> Result<AgentToolResult<Self::Details>, String>;
}

/// Context passed into the agent loop.
pub struct AgentContext {
    pub system_prompt: String,
    pub messages: Vec<AgentMessage>,
    pub tools: Vec<Box<dyn AgentTool<Details = Value>>>,
}

/// How much reasoning / thinking a model should perform.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingLevel {
    Off,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
}

/// Identifier for a concrete LLM model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LlmModelId {
    pub provider: String,
    pub model: String,
}

/// Configuration for a single agent loop execution.
pub struct AgentLoopConfig {
    pub model: LlmModelId,
    pub thinking_level: ThinkingLevel,
}

/// Events emitted by the agent during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    AgentStart {
        context: AgentContextSnapshot,
    },
    AgentEnd {
        messages: Vec<AgentMessage>,
    },
    TurnStart {
        turn_id: Uuid,
    },
    TurnEnd {
        turn_id: Uuid,
        /// Tool results produced during this turn.
        tool_results: Vec<AgentToolResult<Value>>,
    },
    MessageStart {
        message: AgentMessage,
    },
    MessageUpdate {
        message: AgentMessage,
    },
    MessageEnd {
        message: AgentMessage,
    },
    ToolExecutionStart {
        tool_call_id: String,
        name: String,
        args_json: String,
    },
    ToolExecutionUpdate {
        tool_call_id: String,
        partial_result: AgentToolResult<Value>,
    },
    ToolExecutionEnd {
        tool_call_id: String,
        result: AgentToolResult<Value>,
        is_error: bool,
    },
}

/// Lightweight snapshot of the agent context for events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContextSnapshot {
    pub system_prompt: String,
    pub messages: Vec<AgentMessage>,
}

/// Full agent state held by the higher level `Agent` wrapper.
pub struct AgentState {
    pub system_prompt: String,
    pub model: LlmModelId,
    pub thinking_level: ThinkingLevel,
    pub tools: Vec<Box<dyn AgentTool<Details = Value>>>,
    pub messages: Vec<AgentMessage>,
    pub is_streaming: bool,
    pub stream_message: Option<AgentMessage>,
    pub pending_tool_calls: Vec<String>,
    pub error: Option<String>,
}

