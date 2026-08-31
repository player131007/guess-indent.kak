use bpaf::{Parser, construct, long, positional};
use guess_indent::guessers::Simple;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Input {
    Stdin,
    File(PathBuf),
}

impl Input {
    pub fn parser() -> impl Parser<Self> {
        positional::<String>("FILE").map(|s| match s.as_str() {
            "-" => Self::Stdin,
            path => Self::File(path.into()),
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Args {
    pub max_lines: Option<usize>,
    pub guesser: Simple,
    pub input: Input,
}

impl Args {
    pub fn parser() -> impl Parser<Self> {
        let max_lines = long("max-lines")
            .help("Maximum number of lines to consider")
            .argument::<usize>("NUM")
            .optional();

        construct!(Self {
            max_lines,
            guesser(Simple::parser()),
            input(Input::parser()),
        })
    }
}
