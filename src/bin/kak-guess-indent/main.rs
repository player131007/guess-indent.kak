mod args;

use anyhow::{Context, Result};
use args::{Args, Input};
use bpaf::Parser;
use either::Either::{Left, Right};
use guess_indent::{GuessIndent, Indentation};
use std::{
    env::args_os,
    fs::File,
    io::{self, BufReader, Write, stdin, stdout},
};

fn main() -> Result<()> {
    // if invoked with no args, print out the kakrc
    if args_os().len() == 1 {
        print!("{}", include_str!("../../../rc/guess-indent.kak"));
        return Ok(());
    }

    let args = Args::parser().to_options().run();

    indent_to_kak(&guess_indent(&args)?, &mut stdout()).context("While sending commands to Kakoune")
}

fn guess_indent(args: &Args) -> Result<Indentation> {
    let contents = match args.input {
        Input::Stdin => Left(stdin().lock()),
        Input::File(ref path) => {
            Right(BufReader::new(File::open(path).with_context(|| {
                format!("While opening file: {}", path.display())
            })?))
        }
    };

    either::for_both!(contents, c => args.guesser.guess_indent(c, args.max_lines))
        .context("While reading input")
}

fn indent_to_kak(indent: &Indentation, f: &mut impl Write) -> io::Result<()> {
    match indent {
        Indentation::Spaces(count) => writeln!(f, "set-option buffer indentwidth {count}"),
        Indentation::Tabs => writeln!(f, "set-option buffer indentwidth 0"),
        Indentation::None => Ok(()),
    }
}
