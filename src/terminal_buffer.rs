//! Buffer for a prefix and value that renders to
//! the terminal.
//!
//! Its primarily responsbility is for converting strings
//! to columns representing Unicode graphemes so that we
//! can handle multi-byte characters correctly.
use anyhow::Result;
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::borrow::Cow;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

type Position = (u16, u16);
type Dimension = (u16, u16);

/// Virtualized view of the string buffer as a
/// series of wrapped rows.
struct Lines {
    /// Total number of lines.
    count: usize,
    /// Current active line where the cursor is.
    current: usize,
    /// Break points for newlines as columns.
    newlines: Vec<usize>,
}

impl Lines {
    /// Determine if the cursor is on the first line of input.
    fn is_first_line(&self) -> bool {
        self.current == 0
    }

    /// Count the number of rows occupied by a prefix and buffer
    /// accounting for wrapping to the terminal width and any newlines
    /// in the prefix or buffer.
    fn count(
        size: &Dimension,
        prefix: &str,
        buffer: &str,
        prefix_cols: usize,
        buffer_cols: usize,
        newlines: &mut Vec<usize>,
    ) -> usize {
        let width = size.0 as usize;

        if prefix_cols + buffer_cols <= width {
            1
        } else {
            let graphemes = UnicodeSegmentation::graphemes(prefix, true)
                .chain(UnicodeSegmentation::graphemes(buffer, true));
            let mut rows = 0;
            for (index, grapheme) in graphemes.enumerate() {
                if grapheme == "\n" || index % width == 0 {
                    rows += 1;
                    newlines.push(index);
                }
            }

            //let buffer_graphemes =
            //UnicodeSegmentation::graphemes(prefix, true)
            //.collect::<Vec<&str>>();

            //let mut rows = (prefix_cols + buffer_cols) / width;
            //if rows % width != 0 {
            //rows += 1;
            //}
            //for c in prefix.chars().chain(buffer.chars()) {
            //if c == '\n' {
            //rows += 1;
            //}
            //}

            if rows == 0 {
                1
            } else {
                rows
            }
        }
    }
}

/// Internal buffer for a string that operates on columns
/// and rows and may include a prefix to the buffer value.
pub struct TerminalBuffer<'a> {
    prefix: &'a str,
    buffer: String,
    prefix_cols: usize,
    buffer_cols: usize,
    echo: Option<char>,
    size: Dimension,
    start_position: Position,
    position: Position,
    lines: Lines,
}

impl<'a> TerminalBuffer<'a> {
    /// Create a new buffer using the given prefix and mask character.
    pub fn new(
        prefix: &'a str,
        size: Dimension,
        position: Position,
        echo: Option<char>,
    ) -> Self {
        let prefix_cols: usize = UnicodeWidthStr::width(prefix);
        let buffer = String::new();

        let mut newlines = Vec::new();
        let count =
            Lines::count(&size, prefix, &buffer, prefix_cols, 0, &mut newlines);
        let lines = Lines {
            current: 0,
            count,
            newlines,
        };

        Self {
            prefix,
            prefix_cols,
            buffer_cols: 0,
            echo,
            start_position: position.clone(),
            position,
            size,
            buffer,
            lines,
        }
    }

    /// Get the underlying buffer.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Get the number of columns for the prefix.
    pub fn prefix_columns(&self) -> usize {
        self.prefix_cols
    }

    /*
    /// Get the number of columns for the buffer.
    pub fn buffer_columns(&self) -> usize {
        self.buffer_cols
    }
    */

    /*
    fn relative(&self) -> (u16, u16) {
        let rel_col = if self.position.0 >= self.start_position.0 {
            self.position.0 - self.start_position.0
        } else {
            0
        };

        let rel_row = if self.position.1 >= self.start_position.1 {
            self.position.1 - self.start_position.1
        } else {
            0
        };

        (rel_col, rel_row)
    }
    */

    pub fn clear_screen<W>(&mut self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        writer.queue(Clear(ClearType::All))?;
        writer.queue(cursor::MoveTo(0, 0))?;
        self.position = (0, 0);
        self.start_position = (0, 0);
        // FIXME: maintain current relative cursor position
        let pos = self.end_pos(&self.buffer);
        self.redraw(writer, pos)?;
        Ok(())
    }

    fn count_rows(&mut self) -> usize {
        let newlines = &mut self.lines.newlines;
        let count = Lines::count(
            &self.size,
            self.prefix,
            &self.buffer,
            self.prefix_cols,
            self.buffer_cols,
            newlines,
        );
        self.lines.count = count;
        count
    }

    /// Get the total column width for the prefix and buffer.
    pub fn columns(&self) -> usize {
        self.prefix_cols + self.buffer_cols
    }

    /// Set the terminal size.
    pub fn set_size(&mut self, size: Dimension) {
        self.size = size;
    }

    /// Set the cursor position.
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    /// Update the buffer to a new value.
    fn update<S: AsRef<str>>(&mut self, value: S) {
        self.buffer_cols = UnicodeWidthStr::width(&value.as_ref()[..]);
        self.buffer = value.as_ref().to_string();
    }

    /// Resize the dimensions and update computations.
    pub fn resize(&mut self, size: Dimension) {
        self.set_size(size);
        self.buffer_cols = UnicodeWidthStr::width(&self.buffer[..]);
        self.count_rows();
    }

    /// Push a character onto the buffer and write it but do not flush
    /// the stream.
    ///
    /// This should only be used for control characters and newlines
    /// as it does not respect the masking of visible characters when
    /// echo has been set.
    pub fn push<W>(&mut self, writer: &mut W, c: char) -> Result<()>
    where
        W: Write,
    {
        self.buffer.push(c);
        writer.write(c.to_string().as_bytes())?;
        Ok(())
    }

    /// Get the graphemes for the buffer.
    fn graphemes(&self) -> Vec<&str> {
        UnicodeSegmentation::graphemes(&self.buffer[..], true)
            .collect::<Vec<&str>>()
    }

    /// Erase the word before the cursor.
    pub fn erase_word_before<W>(&mut self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        if !self.buffer.is_empty() {
            let (column, row) = self.position;
            let after_start = column as usize - self.prefix_cols;
            let before = &self.buffer[0..after_start];
            let after = &self.buffer[after_start..];
            let mut words = (before.trim_end()).split_word_bounds();
            words.next_back();
            let mut buffer = words.collect::<Vec<&str>>().join("");
            let new_col: u16 = (self.prefix_cols
                + UnicodeWidthStr::width(&buffer[..]))
            .try_into()?;
            buffer.push_str(after);
            let position = (new_col, row);
            self.refresh(writer, buffer, position)?;
        }
        Ok(())
    }

    /// Erase a number of columns before the cursor.
    pub fn erase_before<W>(
        &mut self,
        writer: &mut W,
        amount: usize,
    ) -> Result<()>
    where
        W: Write,
    {
        self.erase(writer, amount, true)
    }

    /// Erase a number of columns after the cursor.
    pub fn erase_after<W>(
        &mut self,
        writer: &mut W,
        amount: usize,
    ) -> Result<()>
    where
        W: Write,
    {
        self.erase(writer, amount, false)
    }

    /// Erase a number of columns before or after the cursor.
    fn erase<W>(
        &mut self,
        writer: &mut W,
        amount: usize,
        before: bool,
    ) -> Result<()>
    where
        W: Write,
    {
        let graphemes = self.graphemes();
        if graphemes.len() > 0 {
            // Cursor position relative to start of the buffer
            let (column, row) = self.position;
            let (before_end, after_start, new_col) = if before {
                let after_start = column as usize - self.prefix_columns();
                let before_end = if after_start >= amount {
                    after_start - amount
                } else {
                    amount
                };
                let new_col = self.prefix_cols + (after_start - amount);
                (before_end, after_start, new_col)
            } else {
                let before_end = column as usize - self.prefix_columns();
                let after_start = if before_end + amount <= graphemes.len() {
                    before_end + amount
                } else {
                    graphemes.len()
                };
                (before_end, after_start, column as usize)
            };

            let before_range = 0..before_end;
            let after_range = after_start..self.buffer_cols;

            let mut new_buf = String::new();
            new_buf.push_str(&graphemes[before_range].join(""));
            new_buf.push_str(&graphemes[after_range].join(""));

            self.refresh(writer, new_buf, (new_col.try_into()?, row))?;
        }

        Ok(())
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
    fn write_bytes(&self, writer: &mut dyn Write, bytes: &[u8]) -> Result<()> {
        writer.write(bytes)?;
        writer.flush()?;
        Ok(())
    }

    /// Write the prefix and flush the stream.
    pub fn write_prefix<W>(&mut self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.write_bytes(writer, self.prefix.as_bytes())
    }

    /// Redraw the prefix and buffer moving the cursor
    /// to the given position.
    pub fn redraw<W>(&self, writer: &mut W, position: Position) -> Result<()>
    where
        W: Write,
    {
        let (col, row) = position;

        if self.lines.count == 1 {
            writer.queue(cursor::MoveTo(0, row))?;
            writer.queue(Clear(ClearType::CurrentLine))?;
            if self.lines.is_first_line() {
                writer.write(self.prefix.as_bytes())?;
            }
            writer.write(self.visible().as_ref().as_bytes())?;
            writer.queue(cursor::MoveTo(col, row))?;
            writer.flush()?;
        } else {
            writer.queue(cursor::MoveTo(0, self.start_position.1))?;
            writer.queue(Clear(ClearType::CurrentLine))?;
            writer.queue(Clear(ClearType::FromCursorDown))?;
            writer.queue(cursor::MoveTo(0, self.start_position.1))?;

            let mut it = self.lines.newlines.iter().skip(1);
            let breakpoint = it.next();
            let mut buffer = String::new();
            let graphemes = UnicodeSegmentation::graphemes(self.prefix, true)
                .chain(UnicodeSegmentation::graphemes(&self.buffer[..], true));
            for (index, grapheme) in graphemes.enumerate() {
                buffer.push_str(grapheme);
                if let Some(breakpoint) = breakpoint {
                    if index == *breakpoint {
                        writer.write(buffer.as_bytes())?;
                        buffer = String::new();
                        writer.queue(cursor::MoveToNextLine(1))?;
                        writer.flush()?;
                    }
                }
            }

            //for breakpoint in self.lines.newlines.iter().skip(1) {
            //println!("Breakpoint {}", breakpoint);
            //}

            //writer.write(self.prefix.as_bytes())?;
            //writer.write(self.visible().as_ref().as_bytes())?;
            //writer.queue(cursor::MoveTo(col, row))?;
            writer.flush()?;
        }

        Ok(())
    }

    /// Redraw the prefix and buffer moving the cursor
    /// to the given position.
    pub fn refresh<W, S: AsRef<str>>(
        &mut self,
        writer: &mut W,
        buf: S,
        position: (u16, u16),
    ) -> Result<()>
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
        let graphemes = self.graphemes();

        let (col, row) = self.position;
        let pos = if self.lines.is_first_line() {
            col as usize - self.prefix_cols
        } else {
            col as usize
        };

        let char_str = c.to_string();

        // Appending to the end
        let (before, after) = if pos as usize == self.buffer.len() {
            (&graphemes[..], &graphemes[graphemes.len()..])
        } else {
            let before = &graphemes[0..pos as usize];
            let after = &graphemes[pos as usize..];
            (before, after)
        };

        // Prepare new line buffer
        let mut new_buf = String::new();
        new_buf.push_str(&before.join(""));
        new_buf.push_str(&char_str[..]);
        new_buf.push_str(&after.join(""));

        // Store the updated buffer
        self.update(new_buf);

        // We have an updated buffer column count so can
        // calculate the rows
        let num_rows = self.count_rows();

        // Moving on original line
        let new_pos = if num_rows == self.lines.count {
            ((self.prefix_cols + pos + 1) as u16, row)
        // Wrapping on to new line
        } else {
            self.lines.current = num_rows - self.lines.count;
            (
                0,
                (self.start_position.1 as usize + self.lines.current)
                    .try_into()?,
            )
        };

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

impl Into<String> for TerminalBuffer<'_> {
    fn into(self) -> String {
        self.buffer
    }
}
