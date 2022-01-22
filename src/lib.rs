#![deny(missing_docs)]

//! Prompt library for crossterm.
use anyhow::Result;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::Write;
use unicode_width::UnicodeWidthStr;

/// The options to use when creating a prompt.
#[derive(Default)]
pub struct PromptOptions {
    /// Options for password capture.
    pub password: Option<PassWord>,

    /// Capture multiline input.
    ///
    /// Use Ctrl+c or Ctrl+d to exit the prompt.
    pub multiline: Option<MultiLine>,
}

/// The options for password mode.
pub struct PassWord {
    /// Character to echo for each character input.
    ///
    /// Typically used to obscure input for sensitive
    /// data like passwords.
    pub echo: Option<char>,
}

impl Default for PassWord {
    fn default() -> Self {
        Self { echo: Some('*') }
    }
}

/// The options for multiline mode.
#[derive(Default)]
pub struct MultiLine {
    /// Show the prompt for each line of input.
    pub repeat_prompt: bool,
}

/// Show a prompt.
pub fn prompt<'a, S: AsRef<str>>(
    prompt: S,
    writer: &'a mut impl Write,
    options: &PromptOptions,
) -> Result<String> {
    let value = run(prompt.as_ref(), writer, options)?;
    Ok(value)
}

fn run<'a, S: AsRef<str>>(
    prompt: S,
    writer: &'a mut impl Write,
    options: &PromptOptions,
) -> Result<String> {
    enable_raw_mode()?;

    let mut line = String::new();

    let prompt_cols: u16 =
        UnicodeWidthStr::width(prompt.as_ref()).try_into()?;

    // Write bytes to the sink
    let write_bytes = |writer: &mut dyn Write, bytes: &[u8]| -> Result<()> {
        writer.write(bytes)?;
        writer.flush()?;
        Ok(())
    };

    // Write the initial prompt
    write_bytes(writer, prompt.as_ref().as_bytes())?;

    // Compute the last column
    let end_col = |prompt: &str, line: &str| -> Result<u16> {
        let mut all = prompt.to_string();
        all.push_str(line);
        let cols: u16 = UnicodeWidthStr::width(&all[..]).try_into()?;
        Ok(cols)
    };

    // Redraw the current line
    let redraw = |writer: &mut dyn Write, line: &str| -> Result<()> {
        let mut tmp = prompt.as_ref().to_string();
        tmp.push_str(line);
        writer.write(tmp.as_bytes())?;
        writer.flush()?;
        Ok(())
    };

    let write_char =
        |writer: &mut dyn Write, c: char, line: &mut String| -> Result<()> {
            line.push(c);
            if let Some(password) = &options.password {
                if let Some(echo) = &password.echo {
                    write!(writer, "{}", echo)?;
                }
            } else {
                write!(writer, "{}", c)?;
            }
            writer.flush()?;
            Ok(())
        };

    'prompt: loop {
        let (column, row) = cursor::position()?;

        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Enter => {
                    if let Some(multiline) = &options.multiline {
                        line.push('\n');
                        write!(writer, "{}", '\n')?;
                        writer.execute(cursor::MoveTo(0, row + 1))?;
                        if multiline.repeat_prompt {
                            write_bytes(writer, prompt.as_ref().as_bytes())?;
                        } else {
                            writer.execute(Clear(ClearType::CurrentLine))?;
                        }
                    } else {
                        //write!(&mut writer, "{}", '\n')?;
                        writer.execute(cursor::MoveToNextLine(1))?;
                        break 'prompt;
                    }
                }
                KeyCode::Left => {
                    if column > prompt_cols {
                        writer.execute(cursor::MoveTo(column - 1, row))?;
                    }
                }
                KeyCode::Right => {
                    let end = end_col(prompt.as_ref(), &line)?;
                    if column < end {
                        writer.execute(cursor::MoveTo(column + 1, row))?;
                    }
                }
                KeyCode::Backspace => {
                    writer.queue(cursor::MoveTo(0, row))?;
                    writer.queue(Clear(ClearType::CurrentLine))?;

                    let mut chars = line.chars();
                    chars.next_back();
                    let raw_line = chars.as_str().to_string();

                    let updated_line = if let Some(password) = &options.password
                    {
                        if let Some(echo) = &password.echo {
                            let columns = UnicodeWidthStr::width(&line[..]);
                            if columns > 0 {
                                echo.to_string().repeat(columns - 1)
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        raw_line.clone()
                    };
                    redraw(writer, &updated_line)?;
                    line = raw_line;
                }
                KeyCode::Char(c) => {
                    // Handle Ctrl+c and Ctrl+d
                    if (c == 'c'
                        && event.modifiers.intersects(KeyModifiers::CONTROL))
                        || (c == 'd'
                            && event
                                .modifiers
                                .intersects(KeyModifiers::CONTROL))
                    {
                        writer.execute(cursor::MoveToNextLine(1))?;
                        break 'prompt;
                    // Handle Ctrl+l to clear the screen
                    } else if c == 'l'
                        && event.modifiers.intersects(KeyModifiers::CONTROL)
                    {
                        writer.queue(Clear(ClearType::All))?;
                        writer.queue(cursor::MoveTo(0, 0))?;
                        write_bytes(writer, prompt.as_ref().as_bytes())?;
                    // Handle Ctrl+a, go to line start
                    } else if c == 'a'
                        && event.modifiers.intersects(KeyModifiers::CONTROL)
                    {
                        writer.execute(cursor::MoveTo(prompt_cols, row))?;
                    // Handle Ctrl+e, go to line end
                    } else if c == 'e'
                        && event.modifiers.intersects(KeyModifiers::CONTROL)
                    {
                        let end = end_col(prompt.as_ref(), &line)?;
                        writer.execute(cursor::MoveTo(end, row))?;
                    // Print the character
                    } else {
                        write_char(writer, c, &mut line)?;
                    }
                }
                _ => {
                    //println!("{:?}", event);
                }
            },
            Event::Mouse(_event) => {}
            Event::Resize(_width, _height) => {}
        }
    }

    disable_raw_mode()?;
    Ok(line)
}
