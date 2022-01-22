use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions, Required};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions::new_required(Required {
        max_attempts: 3,
        trim: true,
    });
    let value =
        prompt("Enter an empty value 3 times: ", &mut stdout, &options)?;
    if value.is_empty() {
        println!("aborted after 3 attempts");
    } else {
        println!("you entered a value: {}", value);
    }
    Ok(())
}
