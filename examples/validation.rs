use anyhow::Result;

use crossterm_prompt::{prompt, PromptOptions, Validation};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions::new_validation(Validation {
        validate: Box::new(|s| {
            if s == "world" {
                true
            } else {
                println!(r#"invalid value, type "world"!"#);
                false
            }
        }),
    });
    let value = prompt(r#"Enter the word "world": "#, &mut stdout, &options)?;
    println!("Hello, {}!", value);
    Ok(())
}