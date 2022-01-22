//! Options for creating prompts.
use std::borrow::Cow;

/// The options to use when creating a prompt.
#[derive(Default)]
pub struct PromptOptions {
    /// Options for requiring a value.
    pub required: Option<Required>,

    /// Options for password capture.
    pub password: Option<PassWord>,

    /// Options for multiline input.
    ///
    /// Use Ctrl+c or Ctrl+d to exit the prompt.
    pub multiline: Option<MultiLine>,

    /// Options for validating the input.
    pub validation: Option<Validation>,

    /// Options for transforming the value.
    pub transformer: Option<Transformer>,
}

impl PromptOptions {
    /// Create the prompt options for a password.
    pub fn new_password(password: PassWord) -> Self {
        Self {
            password: Some(password),
            multiline: Default::default(),
            required: Default::default(),
            validation: Default::default(),
            transformer: Default::default(),
        }
    }

    /// Create the prompt options for a multiline input.
    pub fn new_multiline(multiline: MultiLine) -> Self {
        Self {
            password: Default::default(),
            multiline: Some(multiline),
            required: Default::default(),
            validation: Default::default(),
            transformer: Default::default(),
        }
    }

    /// Create the prompt options for a required value.
    pub fn new_required(required: Required) -> Self {
        Self {
            password: Default::default(),
            multiline: Default::default(),
            required: Some(required),
            validation: Default::default(),
            transformer: Default::default(),
        }
    }

    /// Create the prompt options for a validation.
    pub fn new_validation(validation: Validation) -> Self {
        Self {
            password: Default::default(),
            multiline: Default::default(),
            required: Default::default(),
            validation: Some(validation),
            transformer: Default::default(),
        }
    }

    /// Create the prompt options for a transformer.
    pub fn new_transformer(transformer: Transformer) -> Self {
        Self {
            password: Default::default(),
            multiline: Default::default(),
            required: Default::default(),
            validation: Default::default(),
            transformer: Some(transformer),
        }
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
