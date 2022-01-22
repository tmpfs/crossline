use anyhow::Result;
use std::borrow::Cow;

use crossterm_prompt::{prompt, PromptOptions, Transformer};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let options = PromptOptions::new().transformer(Transformer {
        transform: Box::new(|s| Cow::Owned(s.trim().to_string())),
    });
    let value = prompt(
        "Enter a value with leading/trailing space: ",
        &mut stdout,
        &options,
    )?;
    println!(r#"value: "{}""#, value);
    Ok(())
}
