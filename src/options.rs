//! Options for creating prompts.
use crate::key_binding::KeyBindings;
use std::borrow::Cow;

#[cfg(feature = "history")]
use crate::history::History;

#[cfg(feature = "history")]
use std::sync::Mutex;

/// The options to use when creating a prompt.
#[derive(Default)]
pub struct PromptOptions {
    /// Key bindings to use for the prompt.
    pub(crate) bindings: KeyBindings,

    /// Options for requiring a value.
    pub(crate) required: Option<Required>,

    /// Options for password capture.
    pub(crate) password: Option<PassWord>,

    /// Options for multiline input.
    ///
    /// Use Ctrl+c or Ctrl+d to exit the prompt.
    pub(crate) multiline: Option<MultiLine>,

    /// Options for validating the input.
    pub(crate) validation: Option<Validation>,

    /// Options for transforming the value.
    pub(crate) transformer: Option<Transformer>,

    /// History implementation.
    #[cfg(any(feature = "history", doc))]
    #[doc(cfg(feature = "history"))]
    pub(crate) history: Option<Box<Mutex<dyn History>>>,
}

impl PromptOptions {
    /// Create new prompt options.
    pub fn new() -> Self {
        Default::default()
    }

    /// Configure key bindings.
    pub fn bindings(mut self, bindings: KeyBindings) -> Self {
        self.bindings = bindings;
        self
    }

    /// Configure password for these options.
    pub fn password(mut self, password: PassWord) -> Self {
        self.password = Some(password);
        self
    }

    /// Configure for multiline input.
    pub fn multiline(mut self, multiline: MultiLine) -> Self {
        self.multiline = Some(multiline);
        self
    }

    /// Configure for a required value.
    pub fn required(mut self, required: Required) -> Self {
        self.required = Some(required);
        self
    }

    /// Configure for validation.
    pub fn validation(mut self, validation: Validation) -> Self {
        self.validation = Some(validation);
        self
    }

    /// Configure with a transformer.
    pub fn transformer(mut self, transformer: Transformer) -> Self {
        self.transformer = Some(transformer);
        self
    }

    #[cfg(any(feature = "history", doc))]
    #[doc(cfg(feature = "history"))]
    /// Configure with a history.
    pub fn history(mut self, history: Box<Mutex<dyn History>>) -> Self {
        self.history = Some(history);
        self
    }
}

/// The options for a required value.
#[derive(Default)]
pub struct Required {
    /// Trim the value before checking it is empty.
    ///
    /// Does not affect the underlying value which may
    /// still contain leading and trailing whitespace.
    pub trim: bool,

    /// Maximum number of attempts before giving up.
    ///
    /// Zero indicates to keep repeating the prompt forever.
    pub max_attempts: u16,
}

/// The options for password mode.
pub struct PassWord {
    /// Character to echo for each character input.
    ///
    /// Default is to print the asterisk ('*').
    pub echo: Option<char>,
}

impl Default for PassWord {
    fn default() -> Self {
        Self { echo: Some('*') }
    }
}

/// The options for multiline mode.
#[derive(Default)]
pub struct MultiLine {
    /// Show the prompt for each line of input.
    pub repeat_prompt: bool,
}

/// The options for validation.
pub struct Validation {
    /// Closure to validate the value.
    ///
    /// When a value is invalid (`false`)
    /// a prompt is automatically displayed again.
    pub validate: Box<dyn Fn(&str) -> bool>,
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            validate: Box::new(|_| true),
        }
    }
}

/// The options for transforming the value.
pub struct Transformer {
    /// Closure to transform the value.
    pub transform: Box<dyn Fn(&str) -> Cow<'_, str>>,
}

impl Default for Transformer {
    fn default() -> Self {
        Self {
            transform: Box::new(|value| Cow::Borrowed(value)),
        }
    }
}
