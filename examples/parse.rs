use std::num::ParseIntError;

use crossterm_prompt::parse;

fn parse_u16(value: &str) -> Result<u16, ParseIntError> {
    value.parse::<u16>()
}

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    let options = Default::default();
    let value =
        parse(r#"Enter a u16 number: "#, &mut stdout, &options, parse_u16)?;
    println!("Number is: {}", value);
    Ok(())
}
