use crate::core::{context::Context, skill::FrontMatter};

pub struct PromptEngine {
    pub context: Context,
}

fn section(title: &str) -> String {
    format!("## {}\n", title)
}

fn subsection(title: &str) -> String {
    format!("### {}\n", title)
}

fn bullet_point(content: &str) -> String {
    format!("- {}\n", content)
}

impl PromptEngine {
    pub fn new(context: Context) -> Self {
        PromptEngine { context }
    }

    pub fn render(&self) -> String {
        let mut prompt = String::new();

        for (i, trajectory) in self.context.trajectories.iter().enumerate() {
            prompt.push_str(&section("Trajectory"));
            prompt.push_str(&subsection(&format!("Step {}", i + 1)));
            if let Some(thought) = &trajectory.thought.content {
                prompt.push_str(&bullet_point(&format!("Thought: {}", thought)));
            }
            prompt.push_str(&bullet_point(&format!("Action: {}", trajectory.action)));
            if let Some(params) = &trajectory.params {
                prompt.push_str(&bullet_point(&format!("Params: {}", params)));
            }
            if let Some(observation) = &trajectory.observation {
                prompt.push_str(&bullet_point(&format!("Observation: {}", observation)));
            }
            prompt.push_str("\n");
        }
        prompt
    }

    pub fn build_system_prompt(base: &str, skills: Vec<FrontMatter>) -> String {
        let mut prompt = base.to_string();
        if skills.is_empty() {
            return prompt;
        }

        prompt.push_str(&section("Available Skills"));
        for skill in skills {
            prompt.push_str(&subsection(&skill.name));
            prompt.push_str(&format!("{}\n\n", skill.description));
        }
        prompt
    }
}
