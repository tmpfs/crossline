use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions::new_required(Default::default());
    let value = prompt("Enter a non-empty value: ", &mut stdout, &options)?;
    println!("value: {}", value);
    Ok(())
}
