use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

    pub fn to_prompt(&self) -> Value {
        let mut content = Vec::new();

        if let Some(ref prompt) = self.system_prompt {
            content.push(format!("## System\n{}", prompt));
        }

        if !self.history.is_empty() {
            let mut history_parts = vec!["## Chat History".to_string()];
            for msg in &self.history {
                let role = match msg.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                };
                let text = msg.content.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| msg.content.to_string());
                history_parts.push(format!("### {}\n{}", role, text));
            }
            content.push(history_parts.join("\n\n"));
        }

        if let Some(ref user_msg) = self.user_message {
            content.push(format!("## User\n{}", user_msg));
        }

        let prompt = content.join("\n\n");
        json!([{"role": "user", "content": prompt}])
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
    fn test_to_prompt_full() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful");
        ctx.push_history(Message::user("old question"));
        ctx.push_history(Message::assistant(json!("old answer")));
        ctx.set_user_message("new question");

        let prompt = ctx.to_prompt();
        let messages = prompt.as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");

        let content = messages[0]["content"].as_str().unwrap();
        assert!(content.contains("## System\nYou are helpful"));
        assert!(content.contains("## User\nold question"));
        assert!(content.contains("## Assistant\nold answer"));
        assert!(content.contains("## User\nnew question"));
    }

    #[test]
    fn test_to_prompt_minimal() {
        let mut ctx = Context::new();
        ctx.set_user_message("hello");

        let prompt = ctx.to_prompt();
        let messages = prompt.as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "## User\nhello");
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
