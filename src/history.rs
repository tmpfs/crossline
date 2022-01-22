//! Support for shell history.

/// Options for history implementations.
pub struct HistoryOptions {
    /// Maximum number of history items.
    maximum_size: u16,
}

impl Default for HistoryOptions {
    fn default() -> Self {
        Self { maximum_size: 1000 }
    }
}

/// Trait for history implementations.
pub trait History {
    /// Get the underlying history items.
    fn items(&self) -> &Vec<String>;

    /// Get the number of items in the history.
    fn len(&self) -> usize;

    /// Determine if this history is empty.
    fn is_empty(&self) -> bool;

    /// Determine if the cursor point to the last entry.
    fn is_last(&self) -> bool;

    /// Remove all the history items and reset the cursor.
    fn clear(&mut self);

    /// Push an item onto this history.
    ///
    /// This moves the cursor to the last item in
    /// the history.
    fn push(&mut self, item: String);

    /// Get the item at the current cursor position.
    fn get(&self) -> Option<&String>;

    /// Move the current cursor position and get an item
    /// at the new position.
    fn move_by(&mut self, amount: i16) -> Option<&String>;

    /// Get the position of the cursor.
    fn position(&self) -> &Option<usize>;

    /// Move the cursor to the previous entry in the history.
    fn previous(&mut self) -> Option<&String>;

    /// Move the cursor to the next entry in the history.
    fn next(&mut self) -> Option<&String>;
}

/// Stores history in memory.
#[derive(Default)]
pub struct MemoryHistory {
    items: Vec<String>,
    options: HistoryOptions,
    cursor: Option<usize>,
}

impl MemoryHistory {
    /// Create a new in-memory history.
    pub fn new(options: HistoryOptions) -> Self {
        Self {
            items: vec![],
            cursor: None,
            options,
        }
    }
}

impl History for MemoryHistory {
    fn items(&self) -> &Vec<String> {
        &self.items
    }

    fn position(&self) -> &Option<usize> {
        &self.cursor
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn is_last(&self) -> bool {
        if let Some(cursor) = self.cursor {
            cursor == self.items.len()
        } else {
            false
        }
    }

    fn clear(&mut self) {
        self.items = vec![];
        self.cursor = None;
    }

    fn get(&self) -> Option<&String> {
        if let Some(cursor) = self.cursor {
            self.items.get(cursor)
        } else {
            None
        }
    }

    fn push(&mut self, item: String) {
        self.items.push(item);

        if self.items.len() > self.options.maximum_size as usize {
            self.items.remove(0);
        }
        self.cursor = Some(self.items.len());
    }

    fn previous(&mut self) -> Option<&String> {
        if let Some(cursor) = self.cursor {
            if cursor > 0 {
                if cursor > self.items.len() - 1 {
                    self.cursor = Some(self.items.len() - 1);
                    self.get()
                } else {
                    self.move_by(-1)
                }
            } else {
                self.cursor = Some(0);
                self.get()
            }
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&String> {
        if let Some(cursor) = self.cursor {
            if cursor < self.items.len() - 1 {
                self.move_by(1)
            } else {
                self.cursor = Some(self.items.len());
                self.get()
            }
        } else {
            None
        }
    }

    fn move_by(&mut self, amount: i16) -> Option<&String> {
        if let Some(cursor) = self.cursor {
            let new_pos = if amount.is_negative() {
                cursor - amount.wrapping_abs() as i16 as usize
            } else {
                cursor + amount as usize
            };
            self.cursor = Some(new_pos);
        }
        self.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_basic() {
        let mut history = MemoryHistory::new(Default::default());
        history.push("foo".to_string());
        assert!(!history.is_empty());
        assert_eq!(1, history.len());

        history.push("baz".to_string());
        assert_eq!(&Some(2), history.position());

        assert_eq!(Some(&("baz".to_string())), history.previous());
        assert_eq!(Some(&("foo".to_string())), history.previous());
        assert_eq!(Some(&("baz".to_string())), history.next());
        assert_eq!(None, history.next());

        assert_eq!(&Some(2), history.position());
        assert_eq!(None, history.get());
    }
}
