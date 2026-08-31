use crate::{GuessIndent, Indentation};
use std::{cell::Cell, num::NonZeroUsize, str::FromStr};

/// A simple indentation detector.
///
/// This works by finding the first thing that looks like a valid indentation
/// and assuming that's the file's indentation.
///
/// This will skip multi-line comments in the file if found, because they often
/// have non-standard indentation.
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Simple {
    /// Space indentations that should be considered.
    pub standard_widths: Vec<NonZeroUsize>,
    /// Optional block comment delimiters, used to skip multiline comments.
    pub block_comment_delimiters: Option<(String, String)>,
}

impl Simple {
    #[must_use]
    pub fn parser() -> impl bpaf::Parser<Self> {
        use bpaf::{Parser, construct, long};
        let standard_widths = long("standard-widths")
            .help("Comma-separated list of valid space indentations")
            .argument::<String>("WIDTHS")
            .parse(|s| {
                s.split(',')
                    .map(NonZeroUsize::from_str)
                    .collect::<Result<Vec<_>, _>>()
            })
            .fallback(Vec::new());

        // if either argument is empty, consider it as having no comments
        // this is because i'm too lazy to check if the variables are empty in kakoune
        let block_comment_start = long("block-comment-start")
            .help("String marking the start of a block comment")
            .argument::<String>("STRING")
            .map(|s| (!s.is_empty()).then_some(s));
        let block_comment_end = long("block-comment-end")
            .help("String marking the end of a block comment")
            .argument::<String>("STRING")
            .map(|s| (!s.is_empty()).then_some(s));

        let block_comment_delimiters = construct!(block_comment_start, block_comment_end)
            .map(|(b, e)| b.zip(e))
            .optional()
            .map(Option::flatten);

        construct!(Self {
            standard_widths,
            block_comment_delimiters
        })
    }
}

impl GuessIndent for Simple {
    fn guess_indent<'a>(&self, lines: impl IntoIterator<Item = impl AsRef<str>>) -> Indentation {
        let state = self
            .block_comment_delimiters
            .as_ref()
            .map_or_else(Default::default, CommentState::with_delimiters);
        for line in lines {
            let line = line.as_ref();
            if !state.in_comment.get() {
                let leading_whitespace = line
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or(line, |idx| &line[..idx]);

                match leading_whitespace {
                    "\t" => {
                        return Indentation::Tabs;
                    }
                    s if !s.is_empty() && s.chars().all(|c| c == ' ') => {
                        // SAFETY: we made sure `s` is not empty in the match arm
                        // also `s.len()` is correct here because it only contains ascii characters
                        let len = unsafe { NonZeroUsize::new_unchecked(s.len()) };
                        if self.standard_widths.contains(&len) {
                            return Indentation::Spaces(len);
                        }
                    }
                    _ => {}
                }
            }

            state.advance_state(line);
        }
        Indentation::None
    }
}

/// tracking whether we're in a block comment
/// for simplicity, only block comments at the start of the line will be considered.
#[derive(Debug, PartialEq, Eq, Default)]
struct CommentState<'a> {
    delimiters: Option<(&'a str, &'a str)>,
    in_comment: Cell<bool>,
}

impl<'a> CommentState<'a> {
    fn with_delimiters<T1, T2>((begin, end): &'a (T1, T2)) -> Self
    where
        T1: AsRef<str>,
        T2: AsRef<str>,
    {
        Self {
            delimiters: Some((begin.as_ref(), end.as_ref())),
            ..Default::default()
        }
    }

    fn advance_state(&self, line: &str) {
        let Some((comment_start, comment_end)) = self.delimiters else {
            return;
        };

        if !self.in_comment.get()
            && let Some((front, back)) = line.split_once(comment_start)
        {
            // the block comment isn't at the start of the line, ignore
            if !front.chars().all(|c| c.is_ascii_whitespace()) {
                return;
            }

            // special case: the comment starts and ends at the same line
            if back.contains(comment_end) {
                return;
            }

            self.in_comment.set(true);
        } else if self.in_comment.get() && line.contains(comment_end) {
            self.in_comment.set(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    const TWO: NonZeroUsize = NonZeroUsize::new(2).expect("two is not zero");
    const FOUR: NonZeroUsize = NonZeroUsize::new(4).expect("four is not zero");
    const EIGHT: NonZeroUsize = NonZeroUsize::new(8).expect("eight is not zero");

    #[test]
    fn guess_tab_indent() {
        let guesser = Simple {
            standard_widths: vec![TWO, FOUR, EIGHT],
            block_comment_delimiters: Some(("/*".into(), "*/".into())),
        };

        let contents = indoc! {"
            fn main() {
                \t// weird indentation
            \t// some comment
              //       these lines
                //     will be
                    // ignored
            }
        "};
        assert_eq!(guesser.guess_indent(contents.lines()), Indentation::Tabs);
    }

    #[test]
    fn guess_space_indent() {
        let guesser = Simple {
            standard_widths: vec![TWO, FOUR, EIGHT],
            block_comment_delimiters: None,
        };

        let contents = indoc! {"
            [section]
            foo = 'bar'
            baz = 'qux'
            numbers = [
              1,
              2,
              3,
              # i forgot what's next
            ]
                # why is there suddenly a line here
        "};
        assert_eq!(
            guesser.guess_indent(contents.lines()),
            Indentation::Spaces(TWO)
        );
    }

    #[test]
    fn guess_no_indent() {
        let guesser = Simple {
            standard_widths: vec![TWO, FOUR, EIGHT],
            block_comment_delimiters: None,
        };

        let contents = indoc! {"
            [section]
            foo = 'bar'
            baz = 'qux'
        "};
        assert_eq!(guesser.guess_indent(contents.lines()), Indentation::None);
    }
}
