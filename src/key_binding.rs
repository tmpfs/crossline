//! Type for declaring key bindings.
use crate::command::Command;
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

type CommandHandler = Box<dyn Fn(&KeyEvent) -> Vec<Command>>;

/// Definition of a key event with associated actions.
struct KeyDefinition {
    pub kind: KeyType,
    pub event: Option<KeyEvent>,
    pub actions: CommandHandler,
}

/// Collection of key bindings.
pub struct KeyBindings {
    bindings: Vec<KeyDefinition>,
}

impl KeyBindings {
    /// Find the actions for the first key definition
    /// that matches the given key event.
    pub fn first(&self, event: &KeyEvent) -> Option<Vec<Command>> {
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
                    KeyCode::Char(c) => vec![Command::WriteChar(c)],
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
                actions: Box::new(|_| vec![Command::AcceptLine]),
            },
            // Left
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![Command::MoveCursorLeft]),
            },
            // Right
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![Command::MoveCursorRight]),
            },
            // Backspace
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![Command::EraseCharacter]),
            },
            #[cfg(any(feature = "history", doc))]
            // Up
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![Command::HistoryPrevious]),
            },
            #[cfg(any(feature = "history", doc))]
            // Down
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }),
                actions: Box::new(|_| vec![Command::HistoryNext]),
            },
            // Ctrl+c
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::AbortPrompt]),
            },
            // Ctrl+d
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::AbortPrompt]),
            },
            // Ctrl+l
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::ClearScreen]),
            },
            // Ctrl+a
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::MoveToLineBegin]),
            },
            // Ctrl+e
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::MoveToLineEnd]),
            },
            // Ctrl+u
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::EraseToLineBegin]),
            },
            // Ctrl+k
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::EraseToLineEnd]),
            },
            // Ctrl+w
            KeyDefinition {
                kind: KeyType::Named,
                event: Some(KeyEvent {
                    code: KeyCode::Char('w'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                actions: Box::new(|_| vec![Command::ErasePreviousWord]),
            },
        ];

        Self { bindings }
    }
}
