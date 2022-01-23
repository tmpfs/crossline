#![deny(missing_docs)]
#![feature(doc_cfg)]
#![feature(thread_id_value)]

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

mod key_binding;
mod options;

#[cfg(any(feature = "panic", doc))]
#[doc(cfg(feature = "panic"))]
mod panic;

#[cfg(feature = "panic")]
pub use panic::{stderr_panic_hook, stdout_panic_hook};

mod terminal_buffer;

pub use key_binding::*;
pub use options::*;
use terminal_buffer::TerminalBuffer;

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
pub fn prompt<'a, S: AsRef<str>, W>(
    prefix: S,
    writer: &'a mut W,
    options: &PromptOptions,
) -> Result<String>
where
    W: Write,
{
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

fn validate<'a, S: AsRef<str>, W>(
    prefix: S,
    writer: &'a mut W,
    options: &PromptOptions,
) -> Result<String>
where
    W: Write,
{
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

fn run<'a, S: AsRef<str>, W>(
    prefix: S,
    writer: &'a mut W,
    options: &PromptOptions,
) -> Result<String>
where
    W: Write,
{
    enable_raw_mode()?;

    let _guard = scopeguard::guard((), |_| {
        let _ = disable_raw_mode();
    });

    let echo = if let Some(password) = &options.password {
        password.echo
    } else {
        None
    };
    let mut buf = TerminalBuffer::new(prefix.as_ref(), echo);

    #[cfg(feature = "history")]
    let mut history_buffer = String::new();

    // Write the initial prefix
    buf.write_prefix(writer)?;

    'prompt: loop {
        let (width, height) = size()?;
        let (column, row) = cursor::position()?;

        buf.set_size((width, height));
        buf.set_position((column, row));

        match read()? {
            Event::Key(event) => {
                if let Some(actions) = options.bindings.first(&event) {
                    for action in actions {
                        match action {
                            KeyAction::WriteChar(c) => {
                                buf.write_char(writer, c)?;
                            }
                            KeyAction::SubmitLine => {
                                if let Some(multiline) = &options.multiline {
                                    buf.push(writer, '\n')?;
                                    writer
                                        .execute(cursor::MoveTo(0, row + 1))?;
                                    if multiline.repeat_prompt {
                                        buf.write_prefix(writer)?;
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
                                        writer.push(buf.buffer().to_string());
                                    }

                                    writer
                                        .execute(cursor::MoveToNextLine(1))?;
                                    break 'prompt;
                                }
                            }
                            KeyAction::MoveCursorLeft => {
                                if column as usize > buf.prefix_columns() {
                                    writer.execute(cursor::MoveTo(
                                        column - 1,
                                        row,
                                    ))?;
                                }
                            }
                            KeyAction::MoveCursorRight => {
                                let position = buf.end_pos(buf.buffer());

                                if column < position.0 {
                                    writer.execute(cursor::MoveTo(
                                        column + 1,
                                        row,
                                    ))?;
                                }
                            }
                            KeyAction::EraseCharacter => {
                                buf.erase_before(writer, 1)?;
                            }
                            KeyAction::AbortPrompt => {
                                writer.execute(cursor::MoveToNextLine(1))?;
                                break 'prompt;
                            }
                            KeyAction::ClearScreen => {
                                writer.queue(Clear(ClearType::All))?;
                                writer.queue(cursor::MoveTo(0, 0))?;
                                buf.write_prefix(writer)?;
                            }
                            KeyAction::MoveToLineBegin => {
                                writer.execute(cursor::MoveTo(
                                    buf.prefix_columns().try_into()?,
                                    row,
                                ))?;
                            }
                            KeyAction::MoveToLineEnd => {
                                let position = buf.end_pos(buf.buffer());
                                writer
                                    .execute(cursor::MoveTo(position.0, row))?;
                            }
                            KeyAction::ErasePreviousWord => {
                                todo!("erase previous word")
                            }
                            #[cfg(feature = "history")]
                            KeyAction::HistoryPrevious => {
                                if let Some(history) = &options.history {
                                    let mut history = history.lock().unwrap();

                                    if history.is_last() {
                                        history_buffer =
                                            buf.buffer().to_string();
                                    }

                                    if let Some(history_line) =
                                        history.previous()
                                    {
                                        let position =
                                            buf.end_pos(&history_line);

                                        buf.refresh(
                                            writer,
                                            history_line,
                                            position,
                                        )?;
                                    }
                                }
                            }
                            #[cfg(feature = "history")]
                            KeyAction::HistoryNext => {
                                if let Some(history) = &options.history {
                                    let mut history = history.lock().unwrap();
                                    if let Some(history_line) = history.next() {
                                        let position =
                                            buf.end_pos(&history_line);
                                        buf.refresh(
                                            writer,
                                            history_line,
                                            position,
                                        )?;
                                    } else {
                                        let position =
                                            buf.end_pos(&history_buffer);

                                        buf.refresh(
                                            writer,
                                            &history_buffer,
                                            position,
                                        )?;
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

    Ok(buf.into())
}
