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

/// Consumes whitespaces.
fn whitespaces(i: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(i)
}

/// Looks for a separator, does not consume it.
/// EOF counts as a separator.
fn peek_separator(i: &str) -> IResult<&str, ()> {
    let chars = " \t\r\n+-*~";
    if i.len() == 0 {
        Ok((i, ()))
    } else {
        value((), peek(one_of(chars)))(i)
    }
}

/// Returns a command keyword parser.
/// The parser will match either `cmd_tag` or `cmd_shorthant` and return `keyword`.
fn command_keyword<'a>(
    cmd_tag: &'a str,
    cmd_shorthand: &'a str,
    keyword: CommandKeyword,
) -> impl Fn(&'a str) -> IResult<&'a str, CommandKeyword> {
    let parser = alt((tag(cmd_tag), tag(cmd_shorthand)));
    let parser = preceded(whitespaces, parser);
    let parser = terminated(parser, peek_separator);
    let parser = value(keyword, parser);
    move |i: &str| parser(i)
}

/// Parses a command.
fn command(i: &str) -> IResult<&str, Command> {
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
        Err(err) => Err(err), // TODO: nice error handling
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;

    #[test]
    fn test_command_keyword() {
        let quit = command_keyword("quit", "q", CommandKeyword::Quit);

        assert_eq!(quit("q"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit("quit"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit(" \t \n\rquit"), Ok(("", CommandKeyword::Quit)));
        assert_eq!(quit("qt"), Err(nom::Err::Error(("t", ErrorKind::OneOf))));
    }

    #[test]
    fn test_command() {
        // Test simple commands
        assert_eq!(command("q"), Ok(("", Command::Quit)));
        assert_eq!(command("quit"), Ok(("", Command::Quit)));
        assert_eq!(command("c"), Ok(("", Command::Clear)));
        assert_eq!(command("clear"), Ok(("", Command::Clear)));
        assert_eq!(command("h"), Ok(("", Command::Help)));
        assert_eq!(command("help"), Ok(("", Command::Help)));
        assert_eq!(command("e"), Ok(("", Command::Edit)));
        assert_eq!(command("edit"), Ok(("", Command::Edit)));

        // Advanced tests
        assert_eq!(command(" \t \n\rquit+ "), Ok(("+ ", Command::Quit)));
        assert_eq!(command("qt"), Err(nom::Err::Error(("qt", ErrorKind::Tag))));
    }
}
