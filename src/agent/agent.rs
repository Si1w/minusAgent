use anyhow::Result;

use crate::agent::llm::LLM;
use crate::core::{Action, Node};
use crate::session::context::Context;
use crate::session::harness::Harness;

pub struct Agent {
    llm: LLM,
    harness: Harness,
    max_iterations: usize,
}

impl Agent {
    pub fn new(llm: LLM, max_iterations: usize) -> Self {
        Agent { llm, harness: Harness, max_iterations }
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<()> {
        let mut iter = 0;
        loop {
            if iter >= self.max_iterations {
                return Ok(());
            }
            let action = self.llm.run(ctx).await?;
            match &action {
                Action::Completed => return Ok(()),
                Action::Execute(_) => {
                    self.harness.run(ctx).await?;
                }
                _ => {}
            }
            iter += 1;
        }
    }
}
