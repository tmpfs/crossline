use anyhow::Result;
use crossterm_prompt::{history::MemoryHistory, shell, PromptOptions};
use std::sync::Mutex;

#[derive(thiserror::Error, Debug)]
enum Error {}

fn main() -> Result<()> {
    crossterm_prompt::stdout_panic_hook();

    let mut stdout = std::io::stdout();
    let history = Box::new(Mutex::new(MemoryHistory::new(Default::default())));
    let options = PromptOptions::new().history(history);

    println!(r#"Welcome, type "q" or "quit" to exit"#);

    shell(
        || "shell> ",
        &mut stdout,
        || &options,
        |command| {
            if let Some(cmd) = command {
                match &cmd[..] {
                    "q" | "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            Ok::<(), Error>(())
        },
    )?;

    Ok(())
}
