use crate::core::node::Action;

#[derive(Debug, Clone)]
pub enum ThoughtType {
    None,
    Planning,
    Solving,
    GoalSetting,
}

#[derive(Clone)]
pub struct Thought {
    pub thought_type: ThoughtType,
    pub content: Option<String>,
}

#[derive(Clone)]
pub struct Trajectory {
    pub thought: Thought,
    pub action: Action,
    pub observation: Option<String>,
    pub answer: Option<String>,
}

#[derive(Clone)]
pub struct Context {
    pub system_prompt: String,
    pub trajectories: Vec<Trajectory>,
}

impl Context {
    pub fn new(system_prompt: String) -> Self {
        Self {
            system_prompt,
            trajectories: Vec::new(),
        }
    }

    pub fn init_trajectory(&mut self, query: String) {
        self.trajectories.push(Trajectory {
            thought: Thought {
                thought_type: ThoughtType::None,
                content: None,
            },
            action: Action::Pending,
            observation: Some(format!("User Query: {}", query)),
            answer: None,
        });
    }

    pub fn log_trajectory(&mut self, thought: Thought, action: Action, observation: Option<String>, answer: Option<String>) {
        self.trajectories.push(Trajectory {
            thought,
            action,
            observation,
            answer,
        });
    }
}
