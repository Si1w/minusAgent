use std::path::PathBuf;

use anyhow::Result;

use crate::core::context::Context;
use crate::core::skill::Skill;
use crate::core::{Action, Node};
use crate::feature::harness::Harness;
use crate::feature::llm::LLM;

pub struct Agent {
    llm: LLM,
    harness: Harness,
    skills_path: PathBuf,
    max_iterations: usize,
}

impl Agent {
    pub fn new(llm: LLM, skills_path: PathBuf, max_iterations: usize) -> Self {
        Agent { llm, harness: Harness, skills_path, max_iterations }
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<()> {
        let mut iter = 0;
        loop {
            if iter >= self.max_iterations {
                println!("Reached maximum iterations without completing the task.");
                return Ok(());
            }
            let action = self.llm.run(ctx).await?;
            match &action {
                Action::Completed => return Ok(()),
                Action::Execute(_) => {
                    self.harness.run(ctx).await?;
                }
                Action::UseSkill(names) => {
                    let observation = Skill::load_instructions(&self.skills_path, names);
                    ctx.set_last_observation(observation);
                }
                _ => {}
            }
            iter += 1;
        }
    }
}
