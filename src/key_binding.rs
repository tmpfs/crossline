//! Type for declaring key bindings.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Wraps a key event to distinguish between named
/// key codes and arbitrary input.
#[derive(Debug, Eq, PartialEq)]
enum KeyType {
    /// A named key event.
    Named,
    /// An arbitrary character.
    Char,
    /// A function key modifier.
    Func,
}

type KeyActionHandler = Box<dyn Fn(&KeyEvent) -> Vec<KeyAction>>;

/// Definition of a key event with associated actions.
struct KeyDefinition {
    pub kind: KeyType,
    pub event: Option<KeyEvent>,
    pub actions: KeyActionHandler,
}

/// Actions that keys may trigger.
#[derive(Debug, Clone, Copy)]
pub enum KeyAction {
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

    /// Go to previous history item.
    HistoryPrevious,

    /// Go to next history item.
    HistoryNext,
}

/// Collection of key bindings.
pub struct KeyBindings {
    bindings: Vec<KeyDefinition>,
}

impl KeyBindings {
    /// Find the actions for the first key definition
    /// that matches the give key event.
    pub fn first(&self, event: &KeyEvent) -> Option<Vec<KeyAction>> {
        let kind = match event.code {
            KeyCode::Char(_) => {
                if event.modifiers.intersects(KeyModifiers::CONTROL)
                    || event.modifiers.intersects(KeyModifiers::ALT)
                {
                    KeyType::Named
                } else {
                    KeyType::Char
                }
            }
            KeyCode::F(_) => KeyType::Func,
            _ => KeyType::Named,
        };

        self.bindings.iter().find_map(|d| {
            if d.kind == kind {
                match kind {
                    KeyType::Named => {
                        if let Some(ev) = &d.event {
                            if ev == event {
                                Some((d.actions)(event))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    KeyType::Char | KeyType::Func => Some((d.actions)(event)),
                }
            } else {
                None
            }
        })
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        let bindings = vec![
            // Char(c)
            KeyDefinition {
                kind: KeyType::Char,
                event: None,
                actions: Box::new(|event| match event.code {
                    KeyCode::Char(c) => vec![KeyAction::WriteChar(c)],
                    _ => unreachable!(),
                }),
            },
            // Enter
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![KeyAction::SubmitLine]),
            },
            // Left
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![KeyAction::MoveCursorLeft]),
            },
            // Right
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![KeyAction::MoveCursorRight]),
            },
            // Backspace
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![KeyAction::EraseCharacter]),
            },
            // Up
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![KeyAction::HistoryPrevious]),
            },
            // Down
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![KeyAction::HistoryNext]),
            },
            // Ctrl+c
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![KeyAction::AbortPrompt]),
            },
            // Ctrl+d
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![KeyAction::AbortPrompt]),
            },
            // Ctrl+l
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![KeyAction::ClearScreen]),
            },
            // Ctrl+a
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![KeyAction::MoveToLineBegin]),
            },
            // Ctrl+e
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![KeyAction::MoveToLineEnd]),
            },
        ];
        Self { bindings }
    }
}
