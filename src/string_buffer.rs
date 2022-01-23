//! Buffer for a prefix and value that renders to 
//! the terminal.
//!
//! Its primarily responsbility is for converting string 
//! code points to columns so that we can handle multi-byte 
//! characters correctly.
//!
use anyhow::Result;
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::borrow::Cow;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

/// Internal buffer for a string that operates on columns
/// and rows and may include a prefix to the buffer value.
pub struct StringBuffer<'a> {
    prefix: &'a str,
    buffer: String,
    prefix_cols: usize,
    buffer_cols: usize,
    echo: Option<char>,
    size: (u16, u16),
    position: (u16, u16),
}

impl<'a> StringBuffer<'a> {
    /// Create a new buffer using the given prefix and mask character.
    pub fn new(prefix: &'a str, echo: Option<char>) -> Self {
        let prefix_cols: usize = UnicodeWidthStr::width(prefix);
        Self {
            prefix,
            prefix_cols,
            buffer: String::new(),
            buffer_cols: 0,
            echo,
            size: (0, 0),
            position: (0, 0),
        }
    }

    /// Get the underlying buffer.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Get a mutable reference to the underlying buffer.
    pub fn buffer_mut(&mut self) -> &mut String {
        &mut self.buffer
    }

    /// Get the number of columns for the prefix.
    pub fn prefix_columns(&self) -> usize {
        self.prefix_cols
    }

    /// Set the terminal size
    pub fn set_size(&mut self, size: (u16, u16)) {
        self.size = size;
    }

    /// Set the cursor position.
    pub fn set_position(&mut self, position: (u16, u16)) {
        self.position = position;
    }

    /// Update the buffer to a new value.
    pub fn update(&mut self, value: String) {
        self.buffer_cols = UnicodeWidthStr::width(&value[..]);
        self.buffer = value;
    }

    /// The number of columns this buffer occupies.
    pub fn columns(&self) -> usize {
        self.prefix_cols + self.buffer_cols
    }

    /// Get a visible representation of the buffer.
    pub fn visible(&'a self) -> Cow<'a, str> {
        if let Some(echo) = &self.echo {
            let masked = echo.to_string().repeat(self.buffer_cols);
            Cow::Owned(masked)
        } else {
            Cow::Borrowed(&self.buffer)
        }
    }

    /// Write bytes to the stream and flush.
    pub fn write_bytes(&self, writer: &mut dyn Write, bytes: &[u8]) -> Result<()> {
        writer.write(bytes)?;
        writer.flush()?;
        Ok(())
    }

    /// Redraw the prefix and buffer moving the cursor
    /// to the given position.
    pub fn redraw<W>(&self, writer: &mut W, position: (u16, u16)) -> Result<()>
    where
        W: Write,
    {
        let (col, row) = position;
        writer.queue(cursor::MoveTo(0, row))?;
        writer.queue(Clear(ClearType::CurrentLine))?;
        writer.write(self.prefix.as_bytes())?;
        writer.write(self.visible().as_ref().as_bytes())?;
        writer.queue(cursor::MoveTo(col, row))?;
        writer.flush()?;
        Ok(())
    }

    /// Redraw the prefix and buffer moving the cursor
    /// to the given position.
    pub fn refresh<W, S: AsRef<str>>(&mut self, writer: &mut W, buf: S, position: (u16, u16)) -> Result<()>
    where
        W: Write,
    {
        self.update(buf.as_ref().to_string());
        self.redraw(writer, position)
    }

    // Write a character to the line.
    pub fn write_char<W>(&mut self, writer: &mut W, c: char) -> Result<()>
    where
        W: Write,
    {
        let (col, row) = self.position;
        let pos = col as usize - self.prefix_cols;
        let char_str = c.to_string();

        // Appending to the end
        let (before, after) = if pos as usize == self.buffer.len() {
            (&self.buffer[..], "")
        } else {
            if pos > 0 {
                let before = &self.buffer[0..pos as usize];
                let after = &self.buffer[pos as usize..];
                (before, after)
            } else {
                ("", "")
            }
        };

        // Prepare new line buffer
        let mut new_buf = String::new();
        new_buf.push_str(before);
        new_buf.push_str(&char_str[..]);
        new_buf.push_str(after);

        // Store the updated buffer
        self.update(new_buf);

        let new_pos = ((self.prefix_cols + pos + 1) as u16, row);
        self.redraw(writer, new_pos)?;

        Ok(())
    }

    // Calculate the end position for a value.
    pub fn end_pos(&self, value: &str) -> (u16, u16) {
        let (_col, row) = self.position;
        let (w, _h) = self.size;
        let remainder = w as usize - self.prefix_cols;
        // Fits without wrapping
        if value.len() < remainder {
            let len = UnicodeWidthStr::width(value);
            let new_col = (self.prefix_cols + len) as u16;
            (new_col, row)
        } else {
            todo!("calculate with long wrapped value");
        }
    }
}

impl Into<String> for StringBuffer<'_> {
    fn into(self) -> String {
        self.buffer
    }
}
