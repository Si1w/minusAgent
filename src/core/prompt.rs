use crate::core::context::Context;

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
            prompt.push_str(&bullet_point(&format!("Action: {:?}", trajectory.action)));
            if let Some(observation) = &trajectory.observation {
                prompt.push_str(&bullet_point(&format!("Observation: {}", observation)));
            }
            prompt.push_str("\n");
        }
        prompt
    }
}
