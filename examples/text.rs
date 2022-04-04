use anyhow::Result;

use crossterm_prompt::prompt;

fn main() -> Result<()> {
    crossterm_prompt::stdout_panic_hook();

    let mut stdout = std::io::stdout();
    let value =
        prompt("What is your name? ", &mut stdout, &Default::default())?;
    if let Some(result) = &value {
        println!("Hello, {}!", result);
    }
    Ok(())
}
