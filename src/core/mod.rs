pub mod agent;
pub mod context;
pub mod guard;
pub mod harness;
pub mod llm;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::context::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success { output: String },
    Failure { error: String },
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Outcome::Success { .. })
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, Outcome::Failure { .. })
    }
}

#[async_trait]
pub trait Node: Send + Sync {
    async fn prep(&mut self, ctx: &Context) -> Outcome;
    async fn exec(&mut self, ctx: &Context) -> Outcome;
    async fn post(&mut self, ctx: &mut Context) -> Outcome;

    async fn run(&mut self, ctx: &mut Context) -> Outcome {
        let prep = self.prep(ctx).await;
        if prep.is_failure() {
            return self.post(ctx).await;
        }

        let exec = self.exec(ctx).await;
        if exec.is_failure() {
            return self.post(ctx).await;
        }

        self.post(ctx).await
    }
}