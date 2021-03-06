use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions};

fn main() -> Result<()> {
    crossterm_prompt::stdout_panic_hook();

    let mut stdout = std::io::stdout();
    let options = PromptOptions::new().multiline(Default::default());
    let value = prompt("Enter multiline text: ", &mut stdout, &options)?;
    println!("value: {}", value);
    Ok(())
}
