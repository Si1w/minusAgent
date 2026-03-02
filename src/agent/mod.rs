pub mod llm;

use anyhow::Result;
use llm::LLM;

use crate::core::{Action, Node};
use crate::core::Context;

pub struct Agent {
    llm: LLM,
    max_iterations: usize,
}

impl Agent {
    pub fn new(llm: LLM, max_iterations: usize) -> Self {
        Agent { llm, max_iterations }
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<()> {
        let mut iter = 0;
        loop {
            if iter >= self.max_iterations {
                return Ok(());
            }
            let action = self.llm.run(ctx).await?;
            if action != Action::Running {
                return Ok(());
            }
            iter += 1;
        }
    }
}

