use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Pending,
    Running,
    Completed,
    Execute(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThoughtType {
    None,
    Planning,
    Solving,
    GoalSetting,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Thought {
    pub thought_type: ThoughtType,
    pub content: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub thought: Thought,
    pub action: Action,
    pub observation: Option<String>,
    pub answer: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
    pub system_prompt: String,
    pub trajectories: Vec<Trajectory>,
}

#[async_trait]
pub trait Node: Send + Sync {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>>;
    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>>;
    async fn post(&mut self, prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action>;

    async fn run(&mut self, ctx: &mut Context) -> Result<Action> {
        let prep_res = self.prep(ctx).await?;
        let exec_res = self.exec(prep_res.clone()).await?;
        self.post(prep_res, exec_res, ctx).await
    }
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