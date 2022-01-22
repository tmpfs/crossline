use anyhow::Result;

use crossterm_prompt::{prompt, MultiLine, PromptOptions};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions {
        password: None,
        multiline: Some(MultiLine {
            repeat_prompt: true,
        }),
    };
    let value = prompt("multiline text> ", &mut stdout, &options)?;
    println!("value: {}", value);
    Ok(())
}
