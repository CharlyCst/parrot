use super::scanner::Token;

pub enum Script {
    Quit,
    Edit,
    Filter(Filter),
    Clear,
    Run(Target),
    Show(Target),
    Help,
}

pub enum Filter {
    Name(String),
    Tag(String),
}

pub enum Target {
    All,
    Selected,
}

pub struct ParserError {
    pub message: String,
}

impl ParserError {
    pub fn new(message: &str) -> ParserError {
        ParserError {
            message: String::from(message),
        }
    }
}

pub struct Parser {
    /// Tokens are assumed to end with Token::EOS
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: vec![Token::EOS],
            cursor: 0,
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Script, ParserError> {
        self.tokens = tokens;
        self.cursor = 0;
        self.parse_command()
    }

    /// Parses a single command for the stream of tokens.
    fn parse_command(&mut self) -> Result<Script, ParserError> {
        let token = self.next();
        match token {
            Token::Quit => {
                if self.is_terminator() {
                    Ok(Script::Quit)
                } else {
                    Err(ParserError::new("Quit takes no argument."))
                }
            }
            Token::Clear => {
                if self.is_terminator() {
                    Ok(Script::Clear)
                } else {
                    Err(ParserError::new("Clear takes no argument."))
                }
            }
            Token::Help => {
                if self.is_terminator() {
                    Ok(Script::Help)
                } else {
                    Err(ParserError::new("Help takes no argument."))
                }
            }
            Token::Filter => {
                let args = self.parse_filter_args()?;
                if self.is_terminator() {
                    Ok(Script::Filter(args))
                } else {
                    Err(ParserError::new("Filter takes only one argument."))
                }
            }
            Token::Show => {
                let target = self.parse_target()?;
                if self.is_terminator() {
                    Ok(Script::Show(target))
                } else {
                    Err(ParserError::new("Show takes one or zero argument."))
                }
            }
            Token::Run => {
                let target = self.parse_target()?;
                if self.is_terminator() {
                    Ok(Script::Run(target))
                } else {
                    Err(ParserError::new("Run takes one or zero argument."))
                }
            }
            Token::Edit => {
                if self.is_terminator() {
                    Ok(Script::Edit)
                } else {
                    Err(ParserError::new("Edit takes no argument."))
                }
            }
            Token::EOS => Err(ParserError::new("Please enter a valid command.")),
            token => Err(ParserError::new(&format!("Unexpected token: {}.", token))),
        }
    }

    /// Parses the arguments of the filter command.
    fn parse_filter_args(&mut self) -> Result<Filter, ParserError> {
        let token = self.next();
        match token {
            Token::Lit(name) => Ok(Filter::Name(name.clone())),
            Token::Sha(tag) => Ok(Filter::Tag(tag.clone())),
            _ => Err(ParserError::new(
                "Filter expects a name or a tag as argument.",
            )),
        }
    }

    /// Parses the target, that is either all ('*') or the current selection if
    /// no argument is provided.
    fn parse_target(&mut self) -> Result<Target, ParserError> {
        if self.is_terminator() {
            Ok(Target::Selected)
        } else {
            let token = self.next();
            match token {
                Token::Star => Ok(Target::All),
                _ => Err(ParserError::new(&format!(
                    "Unexpected argument: {}.",
                    token
                ))),
            }
        }
    }

    /// Returns the next token and advance the cursor.
    fn next(&mut self) -> &Token {
        let token = &self.tokens[self.cursor];
        if token != &Token::EOS {
            self.cursor += 1;
        }
        token
    }

    /// Returns true if the next token is a script terminator.
    /// For now the only terminator is the end of script.
    fn is_terminator(&self) -> bool {
        self.tokens[self.cursor] == Token::EOS
    }
}
