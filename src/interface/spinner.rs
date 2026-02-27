use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossterm::cursor;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;

use crate::core::signal::SPINNER_PAUSE;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let flag = running.clone();
        let msg = message.to_string();

        let handle = tokio::spawn(async move {
            let mut stdout = io::stdout();
            let mut active = false;
            let mut i = 0;

            while flag.load(Ordering::Relaxed) {
                if SPINNER_PAUSE.is_on() {
                    if active {
                        stdout.execute(Clear(ClearType::CurrentLine)).ok();
                        print!("\r");
                        stdout.execute(cursor::Show).ok();
                        stdout.flush().ok();
                        active = false;
                    }
                } else {
                    if !active {
                        stdout.execute(cursor::Hide).ok();
                        active = true;
                    }
                    print!("\r{} {}", FRAMES[i % FRAMES.len()], msg);
                    stdout.flush().ok();
                    i += 1;
                }
                tokio::time::sleep(Duration::from_millis(80)).await;
            }

            if active {
                stdout.execute(Clear(ClearType::CurrentLine)).ok();
                print!("\r");
                stdout.execute(cursor::Show).ok();
                stdout.flush().ok();
            }
        });

        Spinner {
            running,
            handle: Some(handle),
        }
    }

    pub async fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.await.ok();
        }
    }
}
