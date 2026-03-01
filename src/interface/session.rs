use anyhow::Result;

use crate::core::config::Config;
use crate::core::context::Context;
use crate::core::prompt::PromptEngine;
use crate::core::skill::FrontMatter;
use crate::core::Router;
use crate::feature::agent::Agent;
use crate::feature::llm::LLM;

const SYSTEM_PROMPT: &str = include_str!("../instructions/prompts/system_prompt.md");

pub struct Session {
    pub agent: Agent,
    pub ctx: Context,
}

impl Session {
    pub fn new(llm_name: Option<&str>) -> Result<Self> {
        let config = Config::load()?;
        let llm_config = config.get_llm(llm_name)?;
        let llm = LLM::from_config(&llm_config);
        let router = Router::new();
        let skills = FrontMatter::register_all_skills(&router.skills_path());
        let system_prompt = PromptEngine::build_system_prompt(SYSTEM_PROMPT, skills);
        let agent = Agent::new(llm, router.skills_path(), config.agent.max_iterations());
        let ctx = Context::new(system_prompt);
        Ok(Session { agent, ctx })
    }

    pub async fn run(&mut self, query: &str) -> Result<()> {
        self.ctx.init_trajectory(query.to_string());
        self.agent.run(&mut self.ctx).await
    }
}