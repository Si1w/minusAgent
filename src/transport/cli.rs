use std::io::{self, BufRead, Write};

use crate::config::{Config, LLMConfig};
use crate::session::{Event, Session};

/// Interactive CLI REPL transport for the agent.
///
/// Reads user input from stdin, drives the session, and displays
/// progress events and final answers to stdout.
///
/// # Fields
/// - `config`: The live configuration, modifiable via `/config` commands.
/// - `session`: The active conversation session (None if not yet initialized).
pub struct Cli {
    config: Config,
    session: Option<Session>,
}

impl Cli {
    /// Creates a new CLI transport from the given configuration.
    ///
    /// Tries to create a session, but enters the REPL even if it fails
    /// (e.g. missing env var). The user can fix config via `/config`
    /// commands and then `/new` to start a session.
    ///
    /// # Arguments
    /// - `config`: The application configuration.
    pub fn new(config: Config) -> Self {
        let session = match Session::new(&config) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("Warning: {}", e);
                eprintln!("Use /config commands to fix, then /new to start a session.\n");
                None
            }
        };
        Self { config, session }
    }

    /// Runs the interactive REPL loop.
    ///
    /// Reads lines from stdin, dispatches slash commands, and sends
    /// user input to the session for agent processing.
    pub async fn run(&mut self) {
        println!("minusAgent v0.1.0");
        println!("Type /exit to quit, /help for available commands.\n");

        let stdin = io::stdin();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match stdin.lock().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }

            let input = input.trim().to_string();
            if input.is_empty() {
                continue;
            }

            if input.starts_with('/') {
                if !self.handle_command(&input).await {
                    break;
                }
                continue;
            }

            let session = match self.session.as_mut() {
                Some(s) => s,
                None => {
                    println!("No active session. Use /config to set up, then /new to start.");
                    continue;
                }
            };

            let answer = session
                .turn(input, |event| match event {
                    Event::Thinking(content) => {
                        println!("\x1b[2m[thinking] {}\x1b[0m", content);
                    }
                    Event::Executing(command) => {
                        println!("\x1b[33m[executing] {}\x1b[0m", command);
                    }
                    Event::Output(content, success) => {
                        if *success {
                            println!("\x1b[2m{}\x1b[0m", content);
                        } else {
                            println!("\x1b[31m[error] {}\x1b[0m", content);
                        }
                    }
                })
                .await;

            println!("\n{}\n", answer);
        }

        println!("Goodbye!");
    }

    /// Handles slash commands. Returns `false` to exit the REPL.
    async fn handle_command(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.splitn(4, ' ').collect();

        match parts[0] {
            "/exit" => return false,
            "/help" => self.cmd_help(),
            "/skills" => self.cmd_skills(),
            "/new" => self.cmd_new(),
            "/switch" => self.cmd_switch(&parts[1..]),
            "/config" => self.cmd_config(&parts[1..]),
            _ => println!("Unknown command: {}", parts[0]),
        }
        true
    }

    /// Displays available commands.
    fn cmd_help(&self) {
        println!("Commands:");
        println!("  /exit                        Exit the REPL");
        println!("  /new                         Start a new session (fresh context)");
        println!("  /skills                      List available skills");
        println!("  /switch <name>               Switch LLM and rebuild session");
        println!("  /config                      View current configuration");
        println!("  /config set <key> <value>    Set a config field (dotted path)");
        println!("  /config add llm              Add an LLM (interactive)");
        println!("  /config remove llm <name>    Remove an LLM by name");
    }

    /// Lists available skills.
    fn cmd_skills(&self) {
        let skills = match &self.session {
            Some(s) => s.skills(),
            None => {
                println!("No active session.");
                return;
            }
        };
        if skills.is_empty() {
            println!("No skills loaded.");
        } else {
            println!("Available skills:");
            for s in skills {
                println!("  - {}: {}", s.name, s.description);
            }
        }
    }

    /// Starts a new session (fresh context).
    fn cmd_new(&mut self) {
        match Session::new(&self.config) {
            Ok(session) => {
                self.session = Some(session);
                println!("New session started.");
            }
            Err(e) => eprintln!("Failed to create session: {}", e),
        }
    }

    /// Switches to a different LLM by name: promotes it to the top of the
    /// config list, then rebuilds the session agent with context preserved.
    fn cmd_switch(&mut self, args: &[&str]) {
        let name = match args.first() {
            Some(n) => *n,
            None => {
                println!("Usage: /switch <name>");
                println!("Available LLMs:");
                for (i, llm) in self.config.llm.iter().enumerate() {
                    let marker = if i == 0 { " (active)" } else { "" };
                    println!("  - {}{}", llm.name, marker);
                }
                return;
            }
        };

        if let Err(e) = self.config.promote_llm(name) {
            eprintln!("Failed: {}", e);
            return;
        }

        match &mut self.session {
            Some(session) => match session.extend(&self.config) {
                Ok(()) => println!("Switched to '{}'. Session rebuilt.", name),
                Err(e) => eprintln!("Failed to rebuild session: {}", e),
            },
            None => match Session::new(&self.config) {
                Ok(session) => {
                    self.session = Some(session);
                    println!("Switched to '{}'. New session started.", name);
                }
                Err(e) => eprintln!("Failed to create session: {}", e),
            },
        }
    }

    /// Dispatches `/config` subcommands.
    fn cmd_config(&mut self, args: &[&str]) {
        match args.first().copied() {
            None => self.config_view(),
            Some("set") => self.config_set(args),
            Some("add") => self.config_add(args),
            Some("remove") => self.config_remove(args),
            Some(sub) => println!("Unknown config subcommand: {}", sub),
        }
    }

    /// Displays the current configuration as pretty JSON.
    fn config_view(&self) {
        match serde_json::to_string_pretty(&self.config) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Failed to display config: {}", e),
        }
    }

    /// Handles `/config set <key> <value>`.
    /// Extends or creates session after successful change.
    fn config_set(&mut self, args: &[&str]) {
        if args.len() < 3 {
            println!("Usage: /config set <key> <value>");
            return;
        }
        match self.config.set(args[1], args[2]) {
            Ok(()) => {
                println!("Set {} = {}", args[1], args[2]);
                match &mut self.session {
                    Some(session) => match session.extend(&self.config) {
                        Ok(()) => {}
                        Err(e) => eprintln!("Failed to rebuild session: {}", e),
                    },
                    None => match Session::new(&self.config) {
                        Ok(s) => {
                            self.session = Some(s);
                            println!("Session started.");
                        }
                        Err(e) => eprintln!("Session not available: {}", e),
                    },
                }
            }
            Err(e) => eprintln!("Failed: {}", e),
        }
    }

    /// Handles `/config add llm` with interactive prompts.
    fn config_add(&mut self, args: &[&str]) {
        if args.get(1) != Some(&"llm") {
            println!("Usage: /config add llm");
            return;
        }

        let stdin = io::stdin();
        let name = prompt_line(&stdin, "Name: ");
        let model = prompt_line(&stdin, "Model: ");
        let base_url = prompt_line(&stdin, "Base URL: ");
        let api_key_env = prompt_line(&stdin, "API key env var: ");

        let llm = LLMConfig {
            name: name.clone(),
            model,
            base_url,
            api_key_env,
            max_tokens: 4096,
            context_window: 128_000,
        };

        match self.config.add_llm(llm) {
            Ok(()) => println!("LLM '{}' added.", name),
            Err(e) => eprintln!("Failed: {}", e),
        }
    }

    /// Handles `/config remove llm <name>`.
    fn config_remove(&mut self, args: &[&str]) {
        if args.get(1) != Some(&"llm") || args.get(2).is_none() {
            println!("Usage: /config remove llm <name>");
            return;
        }
        match self.config.remove_llm(args[2]) {
            Ok(()) => println!("LLM '{}' removed.", args[2]),
            Err(e) => eprintln!("Failed: {}", e),
        }
    }
}

/// Prompts the user for a line of input.
fn prompt_line(stdin: &io::Stdin, prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap_or(0);
    line.trim().to_string()
}
