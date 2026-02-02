use std::io::{self, Write};

use anyhow::Result;

use serde_json::Value;

use crate::context::{Context, Message};
use crate::core::Node;
use crate::cot::ChainOfThought;
use crate::llm::Llm;
use crate::utils::{start_thinking, stop_thinking};

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
                cot.run(ctx).await?;
                stop_thinking(running, handle).await;

                let output = ctx.last_content()
                    .and_then(|v| v["answer"].as_str())
                    .unwrap_or("")
                    .to_string();
                println!("\n{}\n", output);

                // Replace CoT internal response with clean conversation history
                ctx.history.pop();
                ctx.push_history(Message::user(input));
                ctx.push_history(Message::assistant(Value::String(output)));
            } else {
                ctx.push_history(Message::user(input));
                self.llm.run(ctx).await?;
                stop_thinking(running, handle).await;

                if let Some(content) = ctx.last_content().and_then(|v| v.as_str()) {
                    println!("\n{}\n", content);
                }
            }
        }

        Ok(())
    }
}
