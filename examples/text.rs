use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let value =
        prompt("What is your name? ", &mut stdout, &Default::default())?;
    println!("Hello, {}!", value);
    Ok(())
}
