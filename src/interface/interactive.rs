use std::io::{self, Write};

use anyhow::Result;
use serde_json::Value;

use crate::core::{Context, Message, prompt};
use crate::feature::cot::ChainOfThought;
use crate::feature::llm::{Llm, StreamCallback};
use super::utils::{start_thinking, stop_thinking};

pub struct Interactive {
    llm: Llm,
    cot: Option<ChainOfThought>,
}

impl Interactive {
    pub fn new(llm: Llm, cot: bool) -> Self {
        let cot = if cot {
            Some(ChainOfThought::new(llm.clone()))
        } else {
            None
        };
        Self { llm, cot }
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<()> {
        println!("Interactive mode. Type 'exit' to quit.\n");

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "exit" || input == "quit" {
                break;
            }

            if input.is_empty() {
                continue;
            }

            let (running, handle) = start_thinking();

            if let Some(cot) = &mut self.cot {
                ctx.set_user_message(input);
                if let Err(e) = cot.run(ctx).await {
                    stop_thinking(running, handle).await;
                    eprintln!("\nError: {}\n", e);
                    continue;
                }
                stop_thinking(running, handle).await;

                let last = ctx.last_content();
                let output = last
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if output.is_empty() {
                    eprintln!("\nNo answer received. Last response: {:?}\n", last);
                } else {
                    println!("\n{}\n", output);
                }

                // Replace CoT internal response with clean conversation history
                ctx.history.pop();
                ctx.push_history(Message::user(input));
                ctx.push_history(Message::assistant(Value::String(output)));
            } else {
                ctx.push_history(Message::user(input));
                stop_thinking(running, handle).await;

                let callback: StreamCallback = Box::new(|chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().ok();
                });

                let messages = prompt::render(ctx);
                let resp = self.llm.exec_stream(Some(messages), callback).await?;

                // Update context with full response
                if let Some(content) = resp.and_then(|r| r["choices"][0]["message"]["content"].as_str().map(String::from)) {
                    ctx.push_history(Message::assistant(Value::String(content)));
                }
                println!("\n");
            }
        }

        Ok(())
    }
}
