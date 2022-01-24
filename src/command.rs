//! Commands the prompt will execute, typically from a key binding trigger.

/// Commands executed by the prompt.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Abort the prompt (`abort`).
    Abort,
    /// Write the character to the terminal.
    WriteChar(char),
    /// Accept the line (`accept-line`).
    AcceptLine,
    /// Move cursor left (`backward-char`).
    BackwardChar,
    /// Move cursor right (`forward-char`).
    ForwardChar,
    /// Erase the last character (`backward-delete-char`).
    BackwardDeleteChar,
    /// Clear the screen (`clear-screen`).
    ClearScreen,
    /// Move to beginning of the line.
    MoveToLineBegin,
    /// Move to end of the line.
    MoveToLineEnd,

    /// Erase to the beginning of the line.
    BackwardKillLine,

    /// Erase to the end of the line.
    EraseToLineEnd,

    /// Erase the previous word.
    BackwardKillWord,

    // TODO: Ctrl+b
    //BackwardWord,

    // TODO: Ctrl+f
    //ForwardWord,
    /// Go to previous history item (`previous-history`).
    #[cfg(any(feature = "history", doc))]
    #[doc(cfg(feature = "history"))]
    PreviousHistory,

    /// Go to next history item (`next-history`).
    #[cfg(any(feature = "history", doc))]
    #[doc(cfg(feature = "history"))]
    NextHistory,
}
