use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Default)]
pub struct ChatContext {
    messages: Vec<Message>,
    action: Action,
}

impl ChatContext {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            action: Action::default(),
        }
    }

    pub fn with_system(content: impl Into<String>) -> Self {
        Self {
            messages: vec![Message::system(content)],
            action: Action::default(),
        }
    }
}

impl Context for ChatContext {
    fn messages(&self) -> &[Message] {
        &self.messages
    }

    fn push(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    fn extend(&mut self, msgs: Vec<Message>) {
        self.messages.extend(msgs);
    }

    fn action(&self) -> &Action {
        &self.action
    }

    fn set_action(&mut self, action: Action) {
        self.action = action;
    }
}

impl From<Vec<Message>> for ChatContext {
    fn from(messages: Vec<Message>) -> Self {
        Self {
            messages,
            action: Action::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

impl From<Message> for Value {
    fn from(msg: Message) -> Value {
        json!(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Continue,
    Stop,
    CallTool(String),
}

impl Default for Action {
    fn default() -> Self {
        Action::Continue
    }
}

pub trait Context: Send + Sync {
    fn messages(&self) -> &[Message];
    fn push(&mut self, msg: Message);
    fn extend(&mut self, msgs: Vec<Message>);
    fn action(&self) -> &Action;
    fn set_action(&mut self, action: Action);

    fn last(&self) -> Option<&Message> {
        self.messages().last()
    }

    fn last_content(&self) -> Option<&str> {
        self.last().map(|m| m.content.as_str())
    }

    fn to_vec(&self) -> Vec<Value> {
        self.messages().iter().map(|m| json!(m)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_system() {
        let msg = Message::system("You are helpful");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are helpful");
        assert!(msg.name.is_none());
        assert!(msg.tool_call_id.is_none());
    }

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hi there");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Hi there");
    }

    #[test]
    fn test_message_tool() {
        let msg = Message::tool("call_123", "result data");
        assert_eq!(msg.role, Role::Tool);
        assert_eq!(msg.content, "result data");
        assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
    }

    #[test]
    fn test_message_to_value() {
        let msg = Message::user("test");
        let value: Value = msg.into();
        assert_eq!(value["role"], "user");
        assert_eq!(value["content"], "test");
    }

    #[test]
    fn test_action_default() {
        let action = Action::default();
        assert_eq!(action, Action::Continue);
    }

    #[test]
    fn test_action_variants() {
        assert_eq!(Action::Continue, Action::Continue);
        assert_eq!(Action::Stop, Action::Stop);
        assert_eq!(Action::CallTool("foo".into()), Action::CallTool("foo".into()));
        assert_ne!(Action::CallTool("foo".into()), Action::CallTool("bar".into()));
    }

    #[test]
    fn test_chat_context_new() {
        let ctx = ChatContext::new();
        assert!(ctx.messages().is_empty());
        assert_eq!(ctx.action(), &Action::Continue);
    }

    #[test]
    fn test_chat_context_with_system() {
        let ctx = ChatContext::with_system("You are helpful");
        assert_eq!(ctx.messages().len(), 1);
        assert_eq!(ctx.messages()[0].content, "You are helpful");
    }

    #[test]
    fn test_chat_context_from_vec() {
        let messages = vec![
            Message::system("System"),
            Message::user("Hi"),
        ];
        let ctx = ChatContext::from(messages);
        assert_eq!(ctx.messages().len(), 2);
    }

    #[test]
    fn test_chat_context_extend() {
        let mut ctx = ChatContext::with_system("System");
        ctx.extend(vec![
            Message::user("Q1"),
            Message::assistant("A1"),
        ]);
        assert_eq!(ctx.messages().len(), 3);
    }

    #[test]
    fn test_chat_context_action() {
        let mut ctx = ChatContext::new();
        ctx.set_action(Action::CallTool("search".into()));
        assert_eq!(ctx.action(), &Action::CallTool("search".into()));
    }
}
