use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions};

fn main() -> Result<()> {
    crossterm_prompt::stdout_panic_hook();

    let mut stdout = std::io::stdout();
    let options = PromptOptions::new().password(Default::default());
    let value = prompt("Enter a password: ", &mut stdout, &options)?;
    if let Some(result) = &value {
        println!("password: {}", result);
    }
    Ok(())
}
