use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::node::Action;
use super::skill::Skill;

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

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub system_prompt: Option<String>,
    pub user_message: Option<String>,
    pub history: Vec<Message>,
    pub skills: Vec<Skill>,
    pub action: Action,
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
        assert!(ctx.skills.is_empty());
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
