use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions {
        echo: Some('*'),
        multiline: None,
    };
    let value = prompt("Enter a password: ", &mut stdout, &options)?;
    println!("password: {}", value);
    Ok(())
}