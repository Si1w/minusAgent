use anyhow::Result;

use crate::core::config::Config;
use crate::core::context::Context;
use crate::feature::agent::Agent;
use crate::feature::llm::LLM;

const SYSTEM_PROMPT: &str = include_str!("../instructions/system_prompt.md");

pub struct Session {
    pub agent: Agent,
    pub ctx: Context,
}

impl Session {
    pub fn new(llm_name: Option<&str>) -> Result<Self> {
        let config = Config::load()?;
        let llm_config = config.get_llm(llm_name)?;
        let llm = LLM::from_config(&llm_config);
        let agent = Agent::new(llm, config.agent.max_iterations());
        let ctx = Context::new(SYSTEM_PROMPT.to_string());
        Ok(Session { agent, ctx })
    }

    pub async fn run(&mut self, query: &str) -> Result<()> {
        self.ctx.init_trajectory(query.to_string());
        self.agent.run(&mut self.ctx).await
    }
}