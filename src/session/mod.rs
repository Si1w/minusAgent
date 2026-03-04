pub mod harness;

use anyhow::Result;
use harness::Harness;

use crate::config::Config;
use crate::agent::Agent;
use crate::agent::llm::LLM;
use crate::core::{Action, Context, Node};
use crate::skill::SkillRegistry;

const SYSTEM_PROMPT: &str = include_str!("../prompt/system_prompt.md");

pub struct Session {
    pub agent: Agent,
    pub ctx: Context,
    harness: Harness,
    skills: SkillRegistry,
}

impl Session {
    pub fn new(llm_name: Option<&str>) -> Result<Self> {
        let config = Config::load()?;
        let llm_config = config.get_llm(llm_name)?;
        let llm = LLM::from_config(&llm_config);
        let agent = Agent::new(llm, config.agent.max_iterations());
        let skills = SkillRegistry::new();

        let mut system_prompt = SYSTEM_PROMPT.to_string();
        if let Some(skills_prompt) = skills.metadata_prompt() {
            system_prompt.push_str("\n\n");
            system_prompt.push_str(&skills_prompt);
        }
        let ctx = Context::new(system_prompt);

        Ok(Session { agent, ctx, harness: Harness, skills })
    }

    pub async fn query(&mut self, input: &str) -> Result<()> {
        self.ctx.init_trajectory(input.to_string());

        loop {
            self.agent.run(&mut self.ctx).await?;

            match self.ctx.trajectories.last().map(|t| &t.action) {
                Some(Action::Completed) => {
                    if let Some(answer) = self.ctx.trajectories.last().and_then(|t| t.answer.as_ref()) {
                        println!("{}", answer);
                    } else {
                        println!("Task completed");
                    }
                    break;
                }
                Some(Action::Execute(_)) => {
                    self.harness.run(&mut self.ctx).await?;
                }
                Some(Action::UseSkill(names)) => {
                    let instructions = self.skills.activate(names);
                    self.ctx.set_last_observation(instructions);
                }
                _ => {
                    println!("Failed to complete task");
                    break;
                }
            }
        }
        Ok(())
    }
}
