pub mod guessers;

use std::num::NonZeroUsize;

/// The detected indentation of the given contents.
#[derive(Debug, Eq, PartialEq)]
pub enum Indentation {
    /// The given number of spaces is used for indentation.
    Spaces(NonZeroUsize),
    /// A tab is used for indentation.
    Tabs,
    /// No indentation is detected.
    None,
}

pub trait GuessIndent {
    /// Guess the indentation used in the given contents
    fn guess_indent(&self, lines: impl IntoIterator<Item = impl AsRef<str>>) -> Indentation;
}
