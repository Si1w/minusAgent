pub mod agent;
pub mod context;
pub mod harness;
pub mod llm;
pub mod prompt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::context::Context;

/// Control flow action signal used throughout the pipeline.
///
/// Returned by `Node::post()` to determine the next step in the agent loop.
///
/// # Variants
/// - `UseSkill`: Load one or more skills by name, observe results, and loop.
/// - `Execute`: Run a shell command via the harness.
/// - `Continue`: Pure thinking step, loop again.
/// - `Completed`: Task is done, return the answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    UseSkill { skills: Vec<String> },
    Execute { command: String },
    Continue,
    Completed { answer: String },
}

/// A unit of work driven through a chained prep → exec → post pipeline.
///
/// - `prep(shared)`: Read and preprocess data from shared store.
/// - `exec(prep_res)`: Pure compute (LLM calls, APIs). No access to shared.
/// - `post(shared, prep_res, exec_res)`: Write results back to shared, return action for flow control.
///
/// The default `run` short-circuits on failure at prep or exec, returning `Action::Continue`.
#[async_trait]
pub trait Node: Send + Sync {
    /// Read and preprocess data from shared store.
    async fn prep(&mut self, shared: &Context) -> Result<Value, String>;

    /// Execute compute logic. No access to shared. Must be idempotent if retries are enabled.
    async fn exec(&mut self, prep_res: Value) -> Result<Value, String>;

    /// Postprocess and write data back to shared. Returns an action for flow control.
    async fn post(&mut self, shared: &mut Context, prep_res: Value, exec_res: Value) -> Action;

    /// Runs the full prep → exec → post pipeline.
    async fn run(&mut self, shared: &mut Context) -> Action {
        let prep_res = match self.prep(shared).await {
            Ok(v) => v,
            Err(e) => return Action::Completed { answer: e },
        };
        let exec_res = match self.exec(prep_res.clone()).await {
            Ok(v) => v,
            Err(e) => return Action::Completed { answer: e },
        };
        self.post(shared, prep_res, exec_res).await
    }
}