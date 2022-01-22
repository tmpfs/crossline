#![deny(missing_docs)]
#![feature(doc_cfg)]

//! Prompt library for crossterm.
use anyhow::Result;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::borrow::Cow;
use std::error::Error;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

mod options;
pub use options::*;

#[cfg(any(feature = "history", doc))]
#[doc(cfg(feature = "history"))]
pub mod history;

#[cfg(any(feature = "shell", doc))]
#[doc(cfg(feature = "shell"))]
/// Run an infinite shell prompt.
pub fn shell<'a, P, W, O, E, H>(
    prefix: P,
    writer: &'a mut W,
    options: O,
    handler: H,
) -> Result<()>
where
    P: Fn() -> &'a str,
    W: Write,
    O: Fn() -> &'a PromptOptions,
    E: Error + Send + Sync + 'static,
    H: Fn(String) -> std::result::Result<(), E>,
{
    loop {
        let prompt_prefix = (prefix)();
        let opts = (options)();
        let value = prompt(prompt_prefix, writer, opts)?;
        (handler)(value)?;
    }
}

/// Show a prompt.
pub fn prompt<'a, S: AsRef<str>>(
    prefix: S,
    writer: &'a mut impl Write,
    options: &PromptOptions,
) -> Result<String> {
    let value = if let Some(required) = &options.required {
        let mut value;
        let mut attempts = 0u16;
        loop {
            value = validate(prefix.as_ref(), writer, options)?;
            let check_value = if required.trim {
                value.trim()
            } else {
                &value[..]
            };
            attempts += 1;
            if !check_value.is_empty()
                || (required.max_attempts > 0
                    && attempts >= required.max_attempts)
            {
                break;
            }
        }
        value
    } else {
        validate(prefix.as_ref(), writer, options)?
    };

    #[cfg(feature = "history")]
    if let Some(history) = &options.history {
        let mut writer = history.lock().unwrap();
        writer.push(value.clone());
    }

    Ok(value)
}

/// Show a prompt and parse the value to another type.
pub fn parse<'a, T, W, S: AsRef<str>>(
    prefix: S,
    writer: &'a mut W,
    options: &PromptOptions,
) -> Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: Error + Sync + Send + 'static,
    W: Write,
{
    let value: String = prompt(prefix.as_ref(), writer, options)?;
    let value: T = (&value[..]).parse::<T>()?;
    Ok(value)
}

fn validate<'a, S: AsRef<str>>(
    prompt: S,
    writer: &'a mut impl Write,
    options: &PromptOptions,
) -> Result<String> {
    let mut value = if let Some(validation) = &options.validation {
        let value = run(prompt.as_ref(), writer, options)?;
        if (validation.validate)(&value) {
            value
        } else {
            validate(prompt.as_ref(), writer, options)?
        }
    } else {
        run(prompt.as_ref(), writer, options)?
    };

    if let Some(transformer) = &options.transformer {
        value = match (transformer.transform)(&value) {
            Cow::Borrowed(_) => value,
            Cow::Owned(s) => s,
        }
    }

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
    let redraw = |writer: &mut dyn Write, row: u16,line: &str| -> Result<()> {
        writer.queue(cursor::MoveTo(0, row))?;
        writer.queue(Clear(ClearType::CurrentLine))?;

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
                KeyCode::Up =>
                {
                    #[cfg(feature = "history")]
                    if let Some(history) = &options.history {
                        if let Some(history_line) =
                            history.lock().unwrap().previous()
                        {
                            redraw(writer, row, history_line)?;
                        }
                    }
                }
                KeyCode::Down =>
                {
                    #[cfg(feature = "history")]
                    if let Some(history) = &options.history {
                        if let Some(history_line) =
                            history.lock().unwrap().next()
                        {
                            redraw(writer, row, history_line)?;
                        } else {
                            redraw(writer, row, &line)?;
                        }
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
                    redraw(writer, row, &updated_line)?;
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
