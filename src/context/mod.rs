use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// A context window for LLM calls, composed of:
/// {SysPrompt, Document, Memory, Tools, UserMessage, MessageHistory}
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub system_prompt: Option<String>,
    pub documents: Vec<String>,
    pub memory: Vec<String>,
    pub tools: Vec<Tool>,
    pub user_message: Option<String>,
    pub history: Vec<Message>,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
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
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    pub fn add_document(&mut self, doc: impl Into<String>) {
        self.documents.push(doc.into());
    }

    pub fn add_memory(&mut self, mem: impl Into<String>) {
        self.memory.push(mem.into());
    }

    pub fn add_tool(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.tools.push(Tool { name: name.into(), description: description.into() });
    }

    pub fn set_user_message(&mut self, msg: impl Into<String>) {
        self.user_message = Some(msg.into());
    }

    pub fn push_history(&mut self, msg: Message) {
        self.history.push(msg);
    }

    pub fn last_content(&self) -> Option<&str> {
        self.history.last().map(|m| m.content.as_str())
    }

    pub fn commit_user_message(&mut self) {
        if let Some(msg) = self.user_message.take() {
            self.history.push(Message::user(msg));
        }
    }

    pub fn to_messages(&self) -> Vec<Value> {
        let mut msgs = Vec::new();

        let mut system_parts = Vec::new();
        if let Some(ref prompt) = self.system_prompt {
            system_parts.push(prompt.clone());
        }
        if !self.documents.is_empty() {
            system_parts.push(format!("\n## Documents\n{}", self.documents.join("\n")));
        }
        if !self.memory.is_empty() {
            system_parts.push(format!("\n## Memory\n{}", self.memory.join("\n")));
        }
        if !self.tools.is_empty() {
            let tools_str: Vec<String> = self.tools.iter()
                .map(|t| format!("- {}: {}", t.name, t.description))
                .collect();
            system_parts.push(format!("\n## Tools\n{}", tools_str.join("\n")));
        }
        if !system_parts.is_empty() {
            msgs.push(json!(Message::system(system_parts.join("\n"))));
        }

        for msg in &self.history {
            msgs.push(json!(msg));
        }

        if let Some(ref user_msg) = self.user_message {
            msgs.push(json!(Message::user(user_msg)));
        }

        msgs
    }
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}

impl From<Message> for Value {
    fn from(msg: Message) -> Value {
        json!(msg)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Action {
    #[default]
    Continue,
    Stop,
    CallTool(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_window_new() {
        let ctx = Context::new();
        assert!(ctx.history.is_empty());
        assert!(ctx.system_prompt.is_none());
        assert!(ctx.documents.is_empty());
        assert!(ctx.memory.is_empty());
        assert!(ctx.tools.is_empty());
        assert!(ctx.user_message.is_none());
    }

    #[test]
    fn test_context_window_set_components() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful");
        ctx.add_document("doc1");
        ctx.add_memory("remember this");
        ctx.add_tool("search", "search the web");
        ctx.set_user_message("hello");

        assert_eq!(ctx.system_prompt.as_deref(), Some("You are helpful"));
        assert_eq!(ctx.documents.len(), 1);
        assert_eq!(ctx.memory.len(), 1);
        assert_eq!(ctx.tools.len(), 1);
        assert_eq!(ctx.user_message.as_deref(), Some("hello"));
    }

    #[test]
    fn test_context_window_to_messages_full() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful");
        ctx.add_document("reference doc");
        ctx.add_memory("user prefers short answers");
        ctx.add_tool("search", "search the web");
        ctx.push_history(Message::user("old question"));
        ctx.push_history(Message::assistant("old answer"));
        ctx.set_user_message("new question");

        let msgs = ctx.to_messages();
        assert_eq!(msgs.len(), 4); // system, user(old), assistant(old), user(new)
        assert_eq!(msgs[0]["role"], "system");
        assert!(msgs[0]["content"].as_str().unwrap().contains("You are helpful"));
        assert!(msgs[0]["content"].as_str().unwrap().contains("reference doc"));
        assert!(msgs[0]["content"].as_str().unwrap().contains("user prefers short answers"));
        assert!(msgs[0]["content"].as_str().unwrap().contains("search"));
        assert_eq!(msgs[1]["content"], "old question");
        assert_eq!(msgs[2]["content"], "old answer");
        assert_eq!(msgs[3]["content"], "new question");
    }

    #[test]
    fn test_context_window_to_messages_minimal() {
        let mut ctx = Context::new();
        ctx.set_user_message("hello");

        let msgs = ctx.to_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "hello");
    }

    #[test]
    fn test_commit_user_message() {
        let mut ctx = Context::new();
        ctx.set_user_message("question");
        assert!(ctx.user_message.is_some());
        assert!(ctx.history.is_empty());

        ctx.commit_user_message();
        assert!(ctx.user_message.is_none());
        assert_eq!(ctx.history.len(), 1);
        assert_eq!(ctx.history[0].content, "question");
        assert_eq!(ctx.history[0].role, Role::User);
    }

    #[test]
    fn test_push_history_and_last() {
        let mut ctx = Context::new();
        ctx.push_history(Message::assistant("response"));
        assert_eq!(ctx.last_content(), Some("response"));
    }

    #[test]
    fn test_message_system() {
        let msg = Message::system("You are helpful");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are helpful");
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
    fn test_message_to_value() {
        let msg = Message::user("test");
        let value: Value = msg.into();
        assert_eq!(value["role"], "user");
        assert_eq!(value["content"], "test");
    }
}
