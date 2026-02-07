use serde_json::{json, Value};

use super::context::{Context, Role};

const DEFAULT_INSTRUCTION: &str = "You are a helpful assistant.";
const OUTPUT_FORMAT: &str = r#"
Wrap your entire response in an action tag:
- <continue>your response</continue> if there are more steps to execute
- <stop>your response</stop> if you have the final answer

Example:
<stop>The answer is 42.</stop>"#;

fn header(name: &str) -> String {
    format!("## {}", name)
}

fn sub_header(name: &str) -> String {
    format!("### {}", name)
}

#[derive(Clone)]
pub struct PromptEngine {
    pub context: Context,
    pub structured_output: bool,
}

impl PromptEngine {
    pub fn new(context: Context) -> Self {
        Self { context, structured_output: true }
    }

    pub fn without_structured_output(mut self) -> Self {
        self.structured_output = false;
        self
    }

    pub fn instruction(&self) -> String {
        let base = self.context.system_prompt.as_deref().unwrap_or(DEFAULT_INSTRUCTION);
        if self.structured_output {
            format!("{}\n{}", base, OUTPUT_FORMAT)
        } else {
            base.to_string()
        }
    }

    pub fn render(&self) -> Value {
        let mut sections = Vec::new();

        // System
        sections.push(format!("{}\n{}", header("System"), self.instruction()));

        // Skills
        if !self.context.skills.is_empty() {
            let skills_list: Vec<String> = self.context.skills.iter()
                .map(|s| format!("{}\n{}", sub_header(&s.name), s.description))
                .collect();
            sections.push(format!("{}\n{}", header("Skills"), skills_list.join("\n\n")));
        }

        // Question (first user message in history)
        let question = self.context.history.iter()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.as_str().map(|s| s.to_string()).unwrap_or_else(|| m.content.to_string()));
        if let Some(q) = question {
            sections.push(format!("{}\n{}", header("Question"), q));
        }

        // Chat History
        if !self.context.history.is_empty() {
            let mut history_parts = vec![header("Chat History")];
            for msg in &self.context.history {
                let role = match msg.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                };
                let text = msg.content.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| msg.content.to_string());
                history_parts.push(format!("{}\n{}", sub_header(role), text));
            }
            sections.push(history_parts.join("\n\n"));
        }

        // User
        if let Some(ref q) = self.context.user_message {
            sections.push(format!("{}\n{}", header("User"), q));
        }

        let prompt = sections.join("\n\n");
        json!([{"role": "user", "content": prompt}])
    }
}

pub fn render(ctx: &Context) -> Value {
    PromptEngine::new(ctx.clone()).render()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;
    use crate::core::context::Message;
    use crate::core::skill::{Skill, SkillContext};

    #[test]
    fn test_render_basic() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful.");
        ctx.set_user_message("Hello");

        let engine = PromptEngine::new(ctx);
        let messages = engine.render();
        let content = messages[0]["content"].as_str().unwrap();

        assert!(content.contains("## System\nYou are helpful."));
        assert!(content.contains("## User\nHello"));
    }

    #[test]
    fn test_render_with_history() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful");
        ctx.push_history(Message::user("old question"));
        ctx.push_history(Message::assistant(json!("old answer")));
        ctx.set_user_message("new question");

        let engine = PromptEngine::new(ctx);
        let messages = engine.render();
        let content = messages[0]["content"].as_str().unwrap();

        assert!(content.contains("## System\nYou are helpful"));
        assert!(content.contains("### User\nold question"));
        assert!(content.contains("### Assistant\nold answer"));
        assert!(content.contains("## User\nnew question"));
    }

    #[test]
    fn test_render_with_skills() {
        let mut ctx = Context::new();
        ctx.set_system_prompt("You are helpful");
        ctx.skills.push(Skill {
            name: "search".into(),
            description: "Search the web".into(),
            context: SkillContext::Inline,
            disable_model_invocation: false,
            script: String::new(),
            parameters: None,
        });
        ctx.set_user_message("Find info");

        let engine = PromptEngine::new(ctx);
        let messages = engine.render();
        let content = messages[0]["content"].as_str().unwrap();

        assert!(content.contains("## Skills"));
        assert!(content.contains("### search\nSearch the web"));
    }
}
