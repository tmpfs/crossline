#![deny(missing_docs)]
#![feature(doc_cfg)]

//! Prompt library for crossterm.
use anyhow::{bail, Result};
use crossterm::{
    cursor,
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::borrow::Cow;
use std::error::Error;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

mod key_binding;
mod options;
pub use key_binding::*;
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
    if prefix.as_ref().len() > u16::MAX as usize {
        bail!("prompt prefix is too long");
    }

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
    prefix: S,
    writer: &'a mut impl Write,
    options: &PromptOptions,
) -> Result<String> {
    let mut value = if let Some(validation) = &options.validation {
        let value = run(prefix.as_ref(), writer, options)?;
        if (validation.validate)(&value) {
            value
        } else {
            validate(prefix.as_ref(), writer, options)?
        }
    } else {
        run(prefix.as_ref(), writer, options)?
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
    prefix: S,
    writer: &'a mut impl Write,
    options: &PromptOptions,
) -> Result<String> {
    enable_raw_mode()?;

    let mut line = String::new();

    #[cfg(feature = "history")]
    let mut history_buffer = String::new();

    let prompt_cols: u16 =
        UnicodeWidthStr::width(prefix.as_ref()).try_into()?;

    // Write the initial prompt
    write_bytes(writer, prefix.as_ref().as_bytes())?;

    'prompt: loop {
        let (width, height) = size()?;
        let (column, row) = cursor::position()?;
        match read()? {
            Event::Key(event) => {
                if let Some(actions) = options.bindings.first(&event) {
                    for action in actions {
                        match action {
                            KeyAction::WriteChar(c) => {
                                write_char(
                                    prefix.as_ref(),
                                    options,
                                    writer,
                                    prompt_cols,
                                    (column, row),
                                    &mut line,
                                    c,
                                )?;
                            }
                            KeyAction::SubmitLine => {
                                if let Some(multiline) = &options.multiline {
                                    line.push('\n');
                                    write!(writer, "{}", '\n')?;
                                    writer
                                        .execute(cursor::MoveTo(0, row + 1))?;
                                    if multiline.repeat_prompt {
                                        write_bytes(
                                            writer,
                                            prefix.as_ref().as_bytes(),
                                        )?;
                                    } else {
                                        writer.execute(Clear(
                                            ClearType::CurrentLine,
                                        ))?;
                                    }
                                } else {
                                    #[cfg(feature = "history")]
                                    if let Some(history) = &options.history {
                                        let mut writer =
                                            history.lock().unwrap();
                                        writer.push(line.clone());
                                    }

                                    writer
                                        .execute(cursor::MoveToNextLine(1))?;
                                    break 'prompt;
                                }
                            }
                            KeyAction::MoveCursorLeft => {
                                if column > prompt_cols {
                                    writer.execute(cursor::MoveTo(
                                        column - 1,
                                        row,
                                    ))?;
                                }
                            }
                            KeyAction::MoveCursorRight => {
                                let position = end_pos(
                                    (column, row),
                                    (width, height),
                                    prompt_cols,
                                    &line,
                                );

                                if column < position.0 {
                                    writer.execute(cursor::MoveTo(
                                        column + 1,
                                        row,
                                    ))?;
                                }
                            }
                            KeyAction::EraseCharacter => {
                                let pos = column - prompt_cols;
                                let (raw_line, new_col) = if pos > 0 {
                                    let before = &line[0..pos as usize - 1];
                                    let after = &line[pos as usize..];
                                    let mut s = String::new();
                                    s.push_str(before);
                                    s.push_str(after);
                                    (s, (prompt_cols + pos) - 1)
                                } else {
                                    (line.clone(), column)
                                };

                                let updated_line = if let Some(password) =
                                    &options.password
                                {
                                    if let Some(echo) = &password.echo {
                                        let columns =
                                            UnicodeWidthStr::width(&line[..]);
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
                                redraw(
                                    prefix.as_ref(),
                                    writer,
                                    (new_col, row),
                                    &updated_line,
                                )?;
                                line = raw_line;
                            }
                            KeyAction::AbortPrompt => {
                                writer.execute(cursor::MoveToNextLine(1))?;
                                break 'prompt;
                            }
                            KeyAction::ClearScreen => {
                                writer.queue(Clear(ClearType::All))?;
                                writer.queue(cursor::MoveTo(0, 0))?;
                                write_bytes(
                                    writer,
                                    prefix.as_ref().as_bytes(),
                                )?;
                            }
                            KeyAction::MoveToLineBegin => {
                                writer.execute(cursor::MoveTo(
                                    prompt_cols,
                                    row,
                                ))?;
                            }
                            KeyAction::MoveToLineEnd => {
                                let position = end_pos(
                                    (column, row),
                                    (width, height),
                                    prompt_cols,
                                    &line,
                                );
                                writer
                                    .execute(cursor::MoveTo(position.0, row))?;
                            }
                            #[cfg(feature = "history")]
                            KeyAction::HistoryPrevious => {
                                if let Some(history) = &options.history {
                                    let mut history = history.lock().unwrap();

                                    if history.is_last() {
                                        history_buffer = line.clone();
                                    }

                                    if let Some(history_line) =
                                        history.previous()
                                    {
                                        let position = end_pos(
                                            (column, row),
                                            (width, height),
                                            prompt_cols,
                                            &history_line,
                                        );
                                        redraw(
                                            prefix.as_ref(),
                                            writer,
                                            position,
                                            history_line,
                                        )?;
                                        line = history_line.clone();
                                    }
                                }
                            }
                            #[cfg(feature = "history")]
                            KeyAction::HistoryNext => {
                                if let Some(history) = &options.history {
                                    let mut history = history.lock().unwrap();
                                    if let Some(history_line) = history.next() {
                                        let position = end_pos(
                                            (column, row),
                                            (width, height),
                                            prompt_cols,
                                            &history_line,
                                        );
                                        redraw(
                                            prefix.as_ref(),
                                            writer,
                                            position,
                                            history_line,
                                        )?;
                                        line = history_line.clone();
                                    } else {
                                        let position = end_pos(
                                            (column, row),
                                            (width, height),
                                            prompt_cols,
                                            &history_buffer,
                                        );

                                        redraw(
                                            prefix.as_ref(),
                                            writer,
                                            position,
                                            &history_buffer,
                                        )?;
                                        line = history_buffer.clone();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Event::Mouse(_event) => {}
            Event::Resize(_width, _height) => {}
        }
    }

    disable_raw_mode()?;
    Ok(line)
}

// Redraw the current line
fn redraw<S: AsRef<str>>(
    prefix: S,
    writer: &mut dyn Write,
    position: (u16, u16),
    line: &str,
) -> Result<()> {
    let (col, row) = position;
    writer.queue(cursor::MoveTo(0, row))?;
    writer.queue(Clear(ClearType::CurrentLine))?;
    writer.write(prefix.as_ref().as_bytes())?;
    writer.write(line.as_bytes())?;
    writer.queue(cursor::MoveTo(col, row))?;
    writer.flush()?;
    Ok(())
}

// Calculate the end position for a value.
fn end_pos(
    position: (u16, u16),
    size: (u16, u16),
    prompt_cols: u16,
    value: &str,
) -> (u16, u16) {
    let (_col, row) = position;
    let (w, _h) = size;
    let remainder = w - prompt_cols;
    // Fits without wrapping
    if value.len() < remainder as usize {
        //let len: u16 = value.len().try_into().unwrap();
        let len: u16 = UnicodeWidthStr::width(&value[..]).try_into().unwrap();
        let new_col = prompt_cols + len;
        (new_col, row)
    } else {
        todo!("calculate with long wrapped value");
    }
}

// Write bytes to the sink and flush the output.
fn write_bytes(writer: &mut dyn Write, bytes: &[u8]) -> Result<()> {
    writer.write(bytes)?;
    writer.flush()?;
    Ok(())
}

// Write a character to the line.
fn write_char<S: AsRef<str>>(
    prefix: S,
    options: &PromptOptions,
    writer: &mut dyn Write,
    prompt_cols: u16,
    position: (u16, u16),
    line: &mut String,
    c: char,
) -> Result<()> {
    let (col, row) = position;
    let pos = col - prompt_cols;
    let char_str = c.to_string();

    // Appending to the end
    let (before, after) = if pos as usize == line.len() {
        (&line[..], "")
    } else {
        if pos > 0 {
            let before = &line[0..pos as usize];
            let after = &line[pos as usize..];
            (before, after)
        } else {
            ("", "")
        }
    };

    // Prepare new line buffer
    let mut new_line = String::new();
    new_line.push_str(before);
    new_line.push_str(&char_str[..]);
    new_line.push_str(after);

    let (before, char_str, after) = if let Some(password) = &options.password {
        if let Some(echo) = &password.echo {
            let before_len = UnicodeWidthStr::width(before);
            let char_len = UnicodeWidthStr::width(&char_str[..]);
            let after_len = UnicodeWidthStr::width(after);
            let echo_str = echo.to_string();
            (
                echo_str.repeat(before_len),
                echo_str.repeat(char_len),
                echo_str.repeat(after_len),
            )
        } else {
            // Password enabled but not echoing
            ("".to_string(), "".to_string(), "".to_string())
        }
    } else {
        (before.to_string(), char_str, after.to_string())
    };

    // Write out the line data
    writer.queue(cursor::MoveTo(0, row))?;
    writer.queue(Clear(ClearType::CurrentLine))?;
    writer.write(prefix.as_ref().as_bytes())?;
    writer.write(before.as_bytes())?;
    writer.write(char_str.as_bytes())?;
    writer.write(after.as_bytes())?;
    writer.queue(cursor::MoveTo(prompt_cols + pos + 1, row))?;
    writer.flush()?;

    // Store the updated line buffer
    *line = new_line;

    Ok(())
}
