use serde_json::Value;

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
    pub params: Option<Value>,
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
            params: None,
            observation: Some(format!("User Query: {}", query)),
            answer: None,
        });
    }

    pub fn log_trajectory(&mut self, thought: Thought, action: Action, params: Option<Value>, observation: Option<String>, answer: Option<String>) {
        self.trajectories.push(Trajectory {
            thought,
            action,
            params,
            observation,
            answer,
        });
    }

    pub fn set_last_observation(&mut self, observation: String) {
        if let Some(last) = self.trajectories.last_mut() {
            last.observation = Some(observation);
        }
    }
}
