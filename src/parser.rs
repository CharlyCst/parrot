use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::one_of;
use nom::combinator::{map, peek, value};
use nom::sequence::{preceded, terminated};
use nom::IResult;

#[derive(Debug, Eq, PartialEq, Clone)]
enum CommandKeyword {
    Quit,
    Clear,
    Help,
    Edit,
    Run,
    Show,
    Update,
    Delete,
    Filter,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Target {
    Selected,
    All,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Filter {
    Name(String),
    Tag(String),
    Passed,
    Failed,
    Waiting,
    Deleted,
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
    Filter(Filter),
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
    let chars = " \t\r\n#+-*~";
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
        Err(Error::recoverable(ErrorKind::Nom(i, nom::error::ErrorKind::NoneOf)))
    }
}

/// Parses a name.
fn name(i: &str) -> CResult<&str, &str> {
    let is_name = move |c: char| c.is_alphanumeric() || c == '-' || c == '_';
    take_while1(is_name)(i)
}

/// Parses a hashtag.
fn hashtag(i: &str) -> CResult<&str, &str> {
    preceded(tag("#"), name)(i)
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
        Err(err) => Err(Error::custom_with_backtrace(ErrorKind::UnexpectedArgument(cmd), err)),
    }
}

/// Parses a filter argument.
fn filter_arg(i: &str) -> CResult<&str, Filter> {
    let waiting = value(Filter::Waiting, tag("~"));
    let passed = value(Filter::Passed, tag("+"));
    let failed = value(Filter::Failed, tag("-"));
    let hashtag = map(hashtag, move |t| Filter::Tag(t.to_owned()));
    let name = map(name, move |n| Filter::Name(n.to_owned()));
    let parser = alt((waiting, passed, failed, hashtag, name));
    let parser = preceded(whitespaces, parser);
    match parser(i) {
        Ok(f) => Ok(f),
        Err(err) => Err(Error::custom_with_backtrace(
            ErrorKind::UnexpectedArgument(CommandKeyword::Filter),
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
    let filter = command_keyword("filter", "f", CommandKeyword::Filter);
    let keyword = alt((quit, clear, help, edit, run, show, update, delete, filter));
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
            CommandKeyword::Filter => {
                let (i, f) = filter_arg(i)?;
                no_args_left(i, Command::Filter(f))
            }
        },
        Err(err) => Err(Error::custom_with_backtrace(ErrorKind::UnknownCommand, err)),
    }
}

pub fn parse(input: &str) -> Result<Command, String> {
    match command(input) {
        Ok((_, cmd)) => Ok(cmd),
        Err(err) => {
            let err = match err {
                nom::Err::Incomplete(_) => panic!("Internal error: should use 'complete' version of nom parsers."),
                nom::Err::Error(err) => err,
                nom::Err::Failure(err) => err,
            };
            match err.kind {
                ErrorKind::Nom(_, _) => Err(String::from("Failed to parse command")),
                ErrorKind::UnknownCommand => Err(String::from("Unknown command")),
                ErrorKind::UnexpectedArgument(cmd) => Err(format!("Unexpected argument in {}", cmd)),
                ErrorKind::TooManyArguments(cmd) => Err(format!("Too many arguments in {}", cmd)),
            }
        }
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
            Command::Filter(_) => write!(f, "filter"),
        }
    }
}

impl std::fmt::Display for CommandKeyword {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandKeyword::Quit => write!(f, "quit"),
            CommandKeyword::Clear => write!(f, "clear"),
            CommandKeyword::Help => write!(f, "help"),
            CommandKeyword::Edit => write!(f, "edit"),
            CommandKeyword::Run => write!(f, "run"),
            CommandKeyword::Show => write!(f, "show"),
            CommandKeyword::Update => write!(f, "update"),
            CommandKeyword::Delete => write!(f, "delete"),
            CommandKeyword::Filter => write!(f, "filter"),
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
            nom::Err::Incomplete(_) => {
                panic!("Internal error: parser must use the 'complete' version of nom's combinators.")
            }
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
        let cmd = CommandKeyword::Run;

        // Should succeed
        assert_eq!(target("", cmd.clone()), Ok(("", Target::Selected)));
        assert_eq!(target("  ", cmd.clone()), Ok(("", Target::Selected)));
        assert_eq!(target("*", cmd.clone()), Ok(("", Target::All)));
        assert_eq!(target("  * ", cmd.clone()), Ok((" ", Target::All)));

        // Should return an error
        assert_eq!(
            target("a *", cmd.clone()),
            Err(Error::custom(ErrorKind::UnexpectedArgument(CommandKeyword::Run)))
        )
    }

    #[test]
    fn test_filter_arg() {
        // Should succeed
        assert_eq!(filter_arg("#test"), Ok(("", Filter::Tag(String::from("test")))));
        assert_eq!(filter_arg("test-2"), Ok(("", Filter::Name(String::from("test-2")))));
        assert_eq!(filter_arg("+"), Ok(("", Filter::Passed)));
        assert_eq!(filter_arg("-"), Ok(("", Filter::Failed)));
        assert_eq!(filter_arg("~"), Ok(("", Filter::Waiting)));
        assert_eq!(filter_arg(" #test "), Ok((" ", Filter::Tag(String::from("test")))));

        // Should return an error
        assert_eq!(
            filter_arg("@test"),
            Err(Error::custom(ErrorKind::UnexpectedArgument(CommandKeyword::Filter)))
        );
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
        let ts = Target::Selected;
        let ta = Target::All;

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
        assert_eq!(command("update"), Ok(("", Command::Update(ts.clone()))));
        assert_eq!(command("u*"), Ok(("", Command::Update(ta.clone()))));
        assert_eq!(command("delete"), Ok(("", Command::Delete(ts.clone()))));
        assert_eq!(command("d*"), Ok(("", Command::Delete(ta.clone()))));
        assert_eq!(command("filter-"), Ok(("", Command::Filter(Filter::Failed))));
        assert_eq!(command("f-"), Ok(("", Command::Filter(Filter::Failed))));
        assert_eq!(command("f+"), Ok(("", Command::Filter(Filter::Passed))));
        assert_eq!(command("f~"), Ok(("", Command::Filter(Filter::Waiting))));
        assert_eq!(
            command("f#tag"),
            Ok(("", Command::Filter(Filter::Tag(String::from("tag")))))
        );
        assert_eq!(
            command("f name"),
            Ok(("", Command::Filter(Filter::Name(String::from("name")))))
        );

        // Should return an error
        assert_eq!(command("qt"), Err(Error::custom(ErrorKind::UnknownCommand)));
        assert_eq!(
            command("quit *"),
            Err(Error::custom(ErrorKind::TooManyArguments(Command::Quit)))
        );
        assert_eq!(
            command("run * *"),
            Err(Error::custom(ErrorKind::TooManyArguments(Command::Run(Target::All))))
        );
    }
}
