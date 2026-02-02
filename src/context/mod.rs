use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A context window for LLM calls, composed of:
/// {SysPrompt, UserMessage, MessageHistory, Action}
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub system_prompt: Option<String>,
    pub user_message: Option<String>,
    pub history: Vec<Message>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Value,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    pub fn set_user_message(&mut self, msg: impl Into<String>) {
        self.user_message = Some(msg.into());
    }

    pub fn push_history(&mut self, msg: Message) {
        self.history.push(msg);
    }

    pub fn last_content(&self) -> Option<&Value> {
        self.history.last().map(|m| &m.content)
    }

    pub fn to_messages(&self) -> Vec<Message> {
        let mut msgs = Vec::new();

        if let Some(ref prompt) = self.system_prompt {
            msgs.push(Message::system(prompt));
        }

        msgs.extend(self.history.iter().cloned());

        if let Some(ref user_msg) = self.user_message {
            msgs.push(Message::user(user_msg));
        }

        msgs
    }
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: Value::String(content.into()) }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: Value::String(content.into()) }
    }

    pub fn assistant(content: Value) -> Self {
        Self { role: Role::Assistant, content }
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
    use serde_json::json;

    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = Context::new();
        assert!(ctx.history.is_empty());
        assert!(ctx.system_prompt.is_none());
        assert!(ctx.user_message.is_none());
    }

    #[test]
    fn test_to_messages_full() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful");
        ctx.push_history(Message::user("old question"));
        ctx.push_history(Message::assistant(json!("old answer")));
        ctx.set_user_message("new question");

        let msgs = ctx.to_messages();
        assert_eq!(msgs.len(), 4);
        assert_eq!(msgs[0].role, Role::System);
        assert_eq!(msgs[0].content, json!("You are helpful"));
        assert_eq!(msgs[1].content, json!("old question"));
        assert_eq!(msgs[2].content, json!("old answer"));
        assert_eq!(msgs[3].content, json!("new question"));
    }

    #[test]
    fn test_to_messages_minimal() {
        let mut ctx = Context::new();
        ctx.set_user_message("hello");

        let msgs = ctx.to_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, Role::User);
        assert_eq!(msgs[0].content, json!("hello"));
    }

    #[test]
    fn test_push_history_and_last() {
        let mut ctx = Context::new();
        ctx.push_history(Message::assistant(json!("response")));
        assert_eq!(ctx.last_content(), Some(&json!("response")));
    }

    #[test]
    fn test_message_constructors() {
        let sys = Message::system("sys");
        assert_eq!(sys.role, Role::System);

        let usr = Message::user("usr");
        assert_eq!(usr.role, Role::User);

        let ast = Message::assistant(json!("ast"));
        assert_eq!(ast.role, Role::Assistant);
    }
}
