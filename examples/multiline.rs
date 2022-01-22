use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions {
        password: None,
        multiline: Some(Default::default()),
    };
    let value = prompt("Enter multiline text: ", &mut stdout, &options)?;
    println!("value: {}", value);
    Ok(())
}
