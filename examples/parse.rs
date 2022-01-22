use crossterm_prompt::parse;

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    let options = Default::default();
    let value: u16 = parse(r#"Enter a u16 number: "#, &mut stdout, &options)?;
    println!("Number is: {}", value);
    Ok(())
}
