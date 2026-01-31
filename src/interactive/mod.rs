use std::io::{self, Write};

use anyhow::Result;

use crate::context::{Context, Message};
use crate::core::Node;
use crate::llm::Llm;
use crate::utils::{start_thinking, stop_thinking};

pub struct Interactive {
    llm: Llm,
}

impl Interactive {
    pub fn new(llm: Llm) -> Self {
        Self { llm }
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

            ctx.push_history(Message::user(input));

            let (running, handle) = start_thinking();
            self.llm.run(ctx).await?;
            stop_thinking(running, handle).await;

            if let Some(content) = ctx.last_content() {
                println!("\n{}\n", content);
            }
        }

        Ok(())
    }
}
