//! Commands the prompt will execute, typically from a key binding trigger.

/// Commands executed by the prompt.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Write the character to the terminal.
    WriteChar(char),
    /// Submit the line.
    SubmitLine,
    /// Move cursor left.
    MoveCursorLeft,
    /// Move cursor right.
    MoveCursorRight,
    /// Erase the last character.
    EraseCharacter,
    /// Clear the screen.
    ClearScreen,
    /// Abort the prompt.
    AbortPrompt,
    /// Move to beginning of the line.
    MoveToLineBegin,
    /// Move to end of the line.
    MoveToLineEnd,

    /// Erase to the beginning of the line.
    EraseToLineBegin,

    /// Erase to the end of the line.
    EraseToLineEnd,

    /// Erase the previous word.
    ErasePreviousWord,

    /// Go to previous history item.
    #[cfg(any(feature = "history", doc))]
    #[doc(cfg(feature = "history"))]
    HistoryPrevious,

    /// Go to next history item.
    #[cfg(any(feature = "history", doc))]
    #[doc(cfg(feature = "history"))]
    HistoryNext,
}

