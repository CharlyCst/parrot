#![allow(dead_code)]

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::one_of;
use nom::combinator::{peek, value};
use nom::sequence::{preceded, terminated};
use nom::IResult;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CommandKeyword {
    Quit,
    Clear,
    Help,
    Edit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    Clear,
    Help,
    Edit,
}

#[derive(Debug)]
struct Error<I> {
    pub kind: ErrorKind<I>,
    backtrace: Vec<Error<I>>,
}

#[derive(Debug, PartialEq)]
enum ErrorKind<I> {
    Nom(I, nom::error::ErrorKind),
    UnknownCommand,
    TooManyArgument(Command),
}

/// Custom IResult
type CResult<I, O> = IResult<I, O, Error<I>>;

/// Consumes whitespaces.
fn whitespaces(i: &str) -> CResult<&str, &str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(i)
}

/// Looks for a separator, does not consume it.
/// EOF counts as a separator.
fn peek_separator(i: &str) -> CResult<&str, ()> {
    let chars = " \t\r\n+-*~";
    if i.len() == 0 {
        Ok((i, ()))
    } else {
        value((), peek(one_of(chars)))(i)
    }
}

/// Ensures that no argument remains.
//fn no_args_left(i: &str, cmd: Command) -> IResult<>

/// Returns a command keyword parser.
/// The parser will match either `cmd_tag` or `cmd_shorthant` and return `keyword`.
fn command_keyword<'a>(
    cmd_tag: &'a str,
    cmd_shorthand: &'a str,
    keyword: CommandKeyword,
) -> impl Fn(&'a str) -> CResult<&'a str, CommandKeyword> {
    let parser = alt((tag(cmd_tag), tag(cmd_shorthand)));
    let parser = preceded(whitespaces, parser);
    let parser = terminated(parser, peek_separator);
    let parser = value(keyword, parser);
    move |i: &str| parser(i)
}

/// Parses a command.
fn command(i: &str) -> CResult<&str, Command> {
    let quit = command_keyword("quit", "q", CommandKeyword::Quit);
    let clear = command_keyword("clear", "c", CommandKeyword::Clear);
    let help = command_keyword("help", "h", CommandKeyword::Help);
    let edit = command_keyword("edit", "e", CommandKeyword::Edit);
    let keyword = alt((quit, clear, help, edit));
    match keyword(i) {
        Ok((i, keyword)) => match keyword {
            CommandKeyword::Quit => Ok((i, Command::Quit)),
            CommandKeyword::Clear => Ok((i, Command::Clear)),
            CommandKeyword::Help => Ok((i, Command::Help)),
            CommandKeyword::Edit => Ok((i, Command::Edit)),
        },
        Err(err) => Err(Error::custom_with_backtrace(ErrorKind::UnknownCommand, err)),
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Command::Quit => write!(f, "quit"),
            Command::Clear => write!(f, "clear"),
            Command::Help => write!(f, "help"),
            Command::Edit => write!(f, "edit"),
        }
    }
}

impl<I> Error<I> {
    /// Builds a custom (failure) error.
    fn custom(kind: ErrorKind<I>) -> nom::Err<Self> {
        nom::Err::Failure(Self {
            kind,
            backtrace: Vec::new(),
        })
    }

    /// Builds a custom (failure) error, keeps the backtrace of a previous error.
    fn custom_with_backtrace(kind: ErrorKind<I>, err: nom::Err<Self>) -> nom::Err<Self> {
        let err = match err {
            nom::Err::Incomplete(_) => panic!(
                "Internal error: parser must use the 'complete' version of nom's combinators."
            ),
            nom::Err::Error(err) => err,
            nom::Err::Failure(err) => err,
        };
        let mut backtrace = Vec::with_capacity(err.backtrace.len() + 1);
        backtrace.push(Self {
            kind: err.kind,
            backtrace: Vec::new(),
        });
        backtrace.extend(err.backtrace);
        nom::Err::Failure(Self {
            kind,
            backtrace: Vec::new(),
        })
    }
}

impl<I> nom::error::ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: ErrorKind::Nom(input, kind),
            backtrace: Vec::new(),
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(Self::from_error_kind(input, kind));
        other
    }
}

impl<I> std::cmp::PartialEq for Error<I>
where
    I: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_keyword() {
        let quit = command_keyword("quit", "q", CommandKeyword::Quit);

        // Should succeed
        assert_eq!(quit("q"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit("quit"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit(" \t \n\rquit"), Ok(("", CommandKeyword::Quit)));

        // Should return an error
        if let Ok((i, _)) = quit("qt") {
            panic!(format!("Should have fail matching 'quit', got: {}", i))
        }
    }

    #[test]
    fn test_command() {
        // Should succeed
        assert_eq!(command("q"), Ok(("", Command::Quit)));
        assert_eq!(command("quit"), Ok(("", Command::Quit)));
        assert_eq!(command("c"), Ok(("", Command::Clear)));
        assert_eq!(command("clear"), Ok(("", Command::Clear)));
        assert_eq!(command("h"), Ok(("", Command::Help)));
        assert_eq!(command("help"), Ok(("", Command::Help)));
        assert_eq!(command("e"), Ok(("", Command::Edit)));
        assert_eq!(command("edit"), Ok(("", Command::Edit)));
        assert_eq!(command(" \t \n\rquit "), Ok((" ", Command::Quit)));

        // Should return an error
        assert_eq!(command("qt"), Err(Error::custom(ErrorKind::UnknownCommand)));
        //assert_eq!(command(" \t \n\rquit arg"), Ok(("+ ", Command::Quit)));
    }
}
