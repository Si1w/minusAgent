use async_trait::async_trait;
use tokio::process::Command;

use crate::core::context::Context;
use crate::core::{Node, Outcome};

const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "> /dev/sd",
    "dd if=",
];

/// Execution environment that runs shell commands through the Node pipeline.
///
/// Harness is a pure executor — it receives a command string and runs it
/// as a subprocess via `sh -c`. Skill resolution, command construction,
/// and environment setup happen upstream (Session/Agent) or inside
/// skill instructions.
///
/// Dangerous commands are blocked before execution.
///
/// # Fields
/// - `command`: The shell command string to execute.
/// - `result`: The execution result, captured during exec.
pub struct Harness {
    command: Option<String>,
    result: Option<Outcome>,
}

impl Harness {
    /// Creates a new harness.
    pub fn new() -> Self {
        Self {
            command: None,
            result: None,
        }
    }

    /// Sets the command to execute in the next run.
    ///
    /// # Arguments
    /// - `command`: The shell command string (supports `&&`, `||`, pipes, etc.).
    pub fn set_command(&mut self, command: String) {
        self.command = Some(command);
    }

    /// Returns the result of the last execution.
    pub fn result(&self) -> Option<&Outcome> {
        self.result.as_ref()
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
    async fn prep(&mut self, _ctx: &Context) -> Outcome {
        let command = match &self.command {
            Some(c) => c,
            None => return Outcome::Failure { error: "no command set".to_string() },
        };

        if let Some(pattern) = check_blocked(command) {
            return Outcome::Failure {
                error: format!("blocked command: '{}'", pattern),
            };
        }

        Outcome::Success { output: "ready".to_string() }
    }

    /// Spawns a subprocess to execute the command via `sh -c`.
    async fn exec(&mut self, _ctx: &Context) -> Outcome {
        let command = match &self.command {
            Some(c) => c,
            None => return Outcome::Failure { error: "no command set".to_string() },
        };

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await;

        let result = match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                Outcome::Success { output: stdout }
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                Outcome::Failure { error: stderr }
            }
            Err(e) => {
                Outcome::Failure { error: format!("execution failed: {}", e) }
            }
        };

        self.result = Some(result.clone());
        result
    }

    /// Returns the execution result as the final Outcome.
    async fn post(&mut self, _ctx: &mut Context) -> Outcome {
        self.result.clone().unwrap_or(Outcome::Failure {
            error: "no result captured".to_string(),
        })
    }
}