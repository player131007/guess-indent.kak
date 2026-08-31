mod args;

use anyhow::{Context, Result};
use args::{Args, Input};
use bpaf::Parser;
use guess_indent::{GuessIndent, Indentation};
use itertools::{Either, Itertools};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write, stdin, stdout},
};

fn main() -> Result<()> {
    let args = Args::parser().to_options().run();

    indent_to_kak(&guess_indent(&args)?, &mut stdout())
        .with_context(|| "While sending commands to Kakoune")
}

fn guess_indent(args: &Args) -> Result<Indentation> {
    let lines = match args.input {
        Input::Stdin => Either::Left(stdin().lines()),
        Input::File(ref path) => Either::Right(
            BufReader::new(
                File::open(path)
                    .with_context(|| format!("While opening file: {}", path.display()))?,
            )
            .lines(),
        ),
    };

    let lines = if let Some(count) = args.max_lines {
        Either::Left(lines.take(count))
    } else {
        Either::Right(lines)
    };

    lines
        .process_results(|l| args.guesser.guess_indent(l))
        .with_context(|| "While reading input")
}

fn indent_to_kak(indent: &Indentation, f: &mut impl Write) -> io::Result<()> {
    match indent {
        Indentation::Spaces(count) => writeln!(f, "set-option buffer indentwidth {count}"),
        Indentation::Tabs => writeln!(f, "set-option buffer indentwidth 0"),
        Indentation::None => Ok(()),
    }
}
