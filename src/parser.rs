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
    Run,
    Show,
    Update,
    Delete,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Target {
    Selected,
    All,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Quit,
    Clear,
    Help,
    Edit,
    Run(Target),
    Show(Target),
    Update(Target),
    Delete(Target),
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
    TooManyArguments(Command),
    UnexpectedArgument(CommandKeyword),
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

/// Parses a command terminator.
/// For now the only command terminator is EOF.
fn command_terminator(i: &str) -> CResult<&str, &str> {
    if i.len() == 0 {
        Ok((i, ""))
    } else {
        Err(Error::recoverable(ErrorKind::Nom(
            i,
            nom::error::ErrorKind::NoneOf,
        )))
    }
}

/// Ensures that no arguments remain.
/// Return `cmd` if no arguments are found, a TooManyArgument error otherwise.
fn no_args_left(i: &str, cmd: Command) -> CResult<&str, Command> {
    let (i, _) = whitespaces(i)?;
    match command_terminator(i) {
        Ok((i, _)) => Ok((i, cmd)),
        Err(_) => Err(Error::custom(ErrorKind::TooManyArguments(cmd))),
    }
}

/// Parses a target, that is either no argument or '*'.
/// If no argument is found, the target is assumed to be 'Selected'.
fn target(i: &str, cmd: CommandKeyword) -> CResult<&str, Target> {
    let (i, _) = whitespaces(i)?;
    let selected = value(Target::Selected, command_terminator);
    let all = value(Target::All, tag("*"));
    let target = alt((all, selected));
    let target = preceded(whitespaces, target);
    match target(i) {
        Ok(t) => Ok(t),
        Err(err) => Err(Error::custom_with_backtrace(
            ErrorKind::UnexpectedArgument(cmd),
            err,
        )),
    }
}

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
    let run = command_keyword("run", "r", CommandKeyword::Run);
    let show = command_keyword("show", "s", CommandKeyword::Show);
    let update = command_keyword("update", "u", CommandKeyword::Update);
    let delete = command_keyword("delete", "d", CommandKeyword::Delete);
    let keyword = alt((quit, clear, help, edit, run, show, update, delete));
    match keyword(i) {
        Ok((i, keyword)) => match keyword {
            CommandKeyword::Quit => no_args_left(i, Command::Quit),
            CommandKeyword::Clear => no_args_left(i, Command::Clear),
            CommandKeyword::Help => no_args_left(i, Command::Help),
            CommandKeyword::Edit => no_args_left(i, Command::Edit),
            CommandKeyword::Run => {
                let (i, t) = target(i, CommandKeyword::Run)?;
                no_args_left(i, Command::Run(t))
            }
            CommandKeyword::Show => {
                let (i, t) = target(i, CommandKeyword::Show)?;
                no_args_left(i, Command::Show(t))
            }
            CommandKeyword::Update => {
                let (i, t) = target(i, CommandKeyword::Update)?;
                no_args_left(i, Command::Update(t))
            }
            CommandKeyword::Delete => {
                let (i, t) = target(i, CommandKeyword::Delete)?;
                no_args_left(i, Command::Delete(t))
            }
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
            Command::Run(_) => write!(f, "run"),
            Command::Show(_) => write!(f, "show"),
            Command::Update(_) => write!(f, "update"),
            Command::Delete(_) => write!(f, "delete"),
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

    /// Build a custom (recoverable) error.
    fn recoverable(kind: ErrorKind<I>) -> nom::Err<Self> {
        nom::Err::Error(Self {
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
    fn test_no_args_left() {
        let cmd = Command::Quit;
        let error = Err(Error::custom(ErrorKind::TooManyArguments(cmd.clone())));

        // Should succeed
        assert_eq!(no_args_left("", cmd.clone()), Ok(("", cmd.clone())));
        assert_eq!(no_args_left("    ", cmd.clone()), Ok(("", cmd.clone())));
        assert_eq!(no_args_left(" \t \n", cmd.clone()), Ok(("", cmd.clone())));

        // Should return an error
        assert_eq!(no_args_left("+", cmd.clone()), error);
        assert_eq!(no_args_left("  arg", cmd.clone()), error);
        assert_eq!(no_args_left("#tag ", cmd.clone()), error);
        assert_eq!(no_args_left(" ~ ", cmd.clone()), error);
    }

    #[test]
    fn test_target() {
        //Should succeed
        assert_eq!(target("", CommandKeyword::Run), Ok(("", Target::Selected)));
        assert_eq!(
            target("  ", CommandKeyword::Run),
            Ok(("", Target::Selected))
        );
        assert_eq!(target("*", CommandKeyword::Run), Ok(("", Target::All)));
        assert_eq!(target("  * ", CommandKeyword::Run), Ok((" ", Target::All)));

        // Should return an error
        assert_eq!(
            target("a *", CommandKeyword::Run),
            Err(Error::custom(ErrorKind::UnexpectedArgument(
                CommandKeyword::Run
            )))
        )
    }

    #[test]
    fn test_command_keyword() {
        let quit = command_keyword("quit", "q", CommandKeyword::Quit);

        // Should succeed
        assert_eq!(quit("q"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit("quit"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit(" \t \n\rquit"), Ok(("", CommandKeyword::Quit)));

        // Should return an error
        if let Ok((i, _)) = quit("qt") {
            panic!(format!("Should have failed matching 'quit', got: {}", i));
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
        assert_eq!(command(" \t \n\rquit "), Ok(("", Command::Quit)));
        assert_eq!(command("run"), Ok(("", Command::Run(Target::Selected))));
        assert_eq!(command("run *"), Ok(("", Command::Run(Target::All))));
        assert_eq!(command("r*"), Ok(("", Command::Run(Target::All))));
        assert_eq!(command("show"), Ok(("", Command::Show(Target::Selected))));
        assert_eq!(command("s*"), Ok(("", Command::Show(Target::All))));
        assert_eq!(command("update"), Ok(("", Command::Update(Target::Selected))));
        assert_eq!(command("u*"), Ok(("", Command::Update(Target::All))));
        assert_eq!(command("delete"), Ok(("", Command::Delete(Target::Selected))));
        assert_eq!(command("d*"), Ok(("", Command::Delete(Target::All))));

        // Should return an error
        assert_eq!(command("qt"), Err(Error::custom(ErrorKind::UnknownCommand)));
        assert_eq!(
            command("quit *"),
            Err(Error::custom(ErrorKind::TooManyArguments(Command::Quit)))
        );
        assert_eq!(
            command("run * *"),
            Err(Error::custom(ErrorKind::TooManyArguments(Command::Run(
                Target::All
            ))))
        );
    }
}
