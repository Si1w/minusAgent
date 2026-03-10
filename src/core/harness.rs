use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;

use crate::core::context::Context;
use crate::core::context::Outcome;
use crate::core::{Action, Node};

const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "> /dev/sd",
    "dd if=",
];

/// Execution environment that runs shell commands through the Node pipeline.
///
/// - **prep**: validates the command and checks for blocked patterns.
/// - **exec**: spawns `sh -c` subprocess (pure compute, no shared access).
/// - **post**: passes through the execution result.
///
/// # Fields
/// - `command`: The shell command string to execute.
pub struct Harness {
    command: Option<String>,
}

impl Harness {
    /// Creates a new harness.
    pub fn new() -> Self {
        Self { command: None }
    }

    /// Sets the command to execute in the next run.
    ///
    /// # Arguments
    /// - `command`: The shell command string (supports `&&`, `||`, pipes, etc.).
    pub fn set_command(&mut self, command: String) {
        self.command = Some(command);
    }
}

/// Checks if a command contains any blocked patterns.
///
/// # Arguments
/// - `command`: The shell command string to check.
///
/// # Returns
/// `Some(pattern)` if a blocked pattern is found, `None` otherwise.
fn check_blocked(command: &str) -> Option<&'static str> {
    let normalized = command.to_lowercase();
    BLOCKED_PATTERNS
        .iter()
        .find(|p| normalized.contains(**p))
        .copied()
}

#[async_trait]
impl Node for Harness {
    /// Validates that a command is set and not blocked.
    async fn prep(&mut self, _shared: &Context) -> Result<Value, String> {
        let command = self.command.as_ref().ok_or("no command set")?;

        if let Some(pattern) = check_blocked(command) {
            return Err(format!("blocked command: '{}'", pattern));
        }

        Ok(Value::String(command.clone()))
    }

    /// Spawns a subprocess to execute the command via `sh -c`.
    async fn exec(&mut self, prep_res: Value) -> Result<Value, String> {
        let command = prep_res.as_str().ok_or("prep_res is not a string")?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| format!("execution failed: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(Value::String(stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(stderr)
        }
    }

    /// Writes the execution result as an observation to shared context.
    async fn post(&mut self, shared: &mut Context, _prep_res: Value, exec_res: Value) -> Action {
        let command = self.command.take().unwrap_or_default();
        let stdout = exec_res.as_str().unwrap_or_default().to_string();
        shared.add_observation(command, Outcome::Success { output: stdout });
        Action::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_blocked_detects_rm_rf() {
        assert!(check_blocked("rm -rf /").is_some());
        assert!(check_blocked("rm -rf ./build").is_none());
        assert!(check_blocked("RM -RF /").is_some());
    }

    #[test]
    fn test_check_blocked_detects_mkfs() {
        assert!(check_blocked("mkfs.ext4 /dev/sda1").is_some());
    }

    #[test]
    fn test_check_blocked_detects_dd() {
        assert!(check_blocked("dd if=/dev/zero of=/dev/sda").is_some());
    }

    #[test]
    fn test_check_blocked_allows_safe_commands() {
        assert!(check_blocked("ls -la").is_none());
        assert!(check_blocked("echo hello && cat file.txt").is_none());
        assert!(check_blocked("rm file.txt").is_none());
    }

    #[tokio::test]
    async fn test_run_no_command_set() {
        let mut harness = Harness::new();
        let mut ctx = Context::new();
        let action = harness.run(&mut ctx).await;
        assert!(matches!(action, Action::Completed { .. }));
    }

    #[tokio::test]
    async fn test_run_echo() {
        let mut harness = Harness::new();
        let mut ctx = Context::new();
        harness.set_command("echo hello".to_string());
        let action = harness.run(&mut ctx).await;
        assert!(matches!(action, Action::Continue));
        let last = ctx.messages().last().unwrap();
        if let crate::core::context::Message::Observation { content, .. } = last {
            assert_eq!(content.trim(), "hello");
        } else {
            panic!("expected Observation message");
        }
    }

    #[tokio::test]
    async fn test_run_blocked_command() {
        let mut harness = Harness::new();
        let mut ctx = Context::new();
        harness.set_command("rm -rf /".to_string());
        let action = harness.run(&mut ctx).await;
        assert!(matches!(action, Action::Completed { .. }));
    }

    #[tokio::test]
    async fn test_run_failing_command() {
        let mut harness = Harness::new();
        let mut ctx = Context::new();
        harness.set_command("false".to_string());
        let action = harness.run(&mut ctx).await;
        assert!(matches!(action, Action::Completed { .. }));
    }

    #[tokio::test]
    async fn test_run_chained_commands() {
        let mut harness = Harness::new();
        let mut ctx = Context::new();
        harness.set_command("echo foo && echo bar".to_string());
        let action = harness.run(&mut ctx).await;
        assert!(matches!(action, Action::Continue));
        let last = ctx.messages().last().unwrap();
        if let crate::core::context::Message::Observation { content, .. } = last {
            assert!(content.contains("foo"));
            assert!(content.contains("bar"));
        } else {
            panic!("expected Observation message");
        }
    }

    #[tokio::test]
    async fn test_run_pipe() {
        let mut harness = Harness::new();
        let mut ctx = Context::new();
        harness.set_command("echo hello world | wc -w".to_string());
        let action = harness.run(&mut ctx).await;
        assert!(matches!(action, Action::Continue));
        let last = ctx.messages().last().unwrap();
        if let crate::core::context::Message::Observation { content, .. } = last {
            assert_eq!(content.trim(), "2");
        } else {
            panic!("expected Observation message");
        }
    }
}