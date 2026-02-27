use anyhow::Result;

use crate::core::{context::Context};
use crate::core::{Action, Node};
use crate::feature::harness::Harness;
use crate::feature::llm::LLM;

const MAX_ITERATIONS: usize = 10;

pub struct Agent {
    llm: LLM,
    harness: Harness,
}

impl Agent {
    pub fn new(llm: LLM) -> Self {
        Agent { llm, harness: Harness }
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<()> {
        let mut iter = 0;
        loop {
            if iter >= MAX_ITERATIONS {
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