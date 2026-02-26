use std::process::Stdio;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::core::context::{Context, Thought, ThoughtType};
use crate::core::{Action, Node};

pub struct Harness;

#[async_trait]
impl Node for Harness {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        let cmd = ctx
            .trajectories
            .last()
            .and_then(|t| match &t.action {
                Action::Execute(Some(cmd)) => Some(cmd.clone()),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("no command to execute"))?;
        Ok(Some(json!({ "command": cmd })))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        let cmd = match prep_res.and_then(|v| v["command"].as_str().map(String::from)) {
            Some(cmd) => cmd,
            None => return Ok(None),
        };

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdin(Stdio::null())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = stdout.to_string();
        if !stderr.is_empty() {
            result.push_str(&format!("[stderr] {}", stderr));
        }
        if !output.status.success() {
            result.push_str(&format!("[exit code: {}]", output.status.code().unwrap_or(-1)));
        }
        Ok(Some(json!({ "output": result })))
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        let observation = exec_res
            .and_then(|v| v["output"].as_str().map(String::from))
            .unwrap_or_else(|| "[error] no output".to_string());

        ctx.log_trajectory(
            Thought { thought_type: ThoughtType::None, content: None },
            Action::Pending,
            Some(observation),
            None,
        );
        Ok(Action::Running)
    }


}