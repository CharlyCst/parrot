use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Quit,
    Filter,
    Edit,
    Clear,
    Update,
    Run,
    Show,
    Help,
    Star,
    Sha(String),
    Lit(String),
    EOS, // End of script
}

pub struct Scanner {
    keywords: HashMap<String, Token>,
    current: String,
    tokens: Vec<Token>,
    is_sha: bool,
}

impl Scanner {
    pub fn new() -> Scanner {
        let mut map = HashMap::new();
        map.insert(String::from("quit"), Token::Quit);
        map.insert(String::from("q"), Token::Quit);
        map.insert(String::from("filter"), Token::Filter);
        map.insert(String::from("f"), Token::Filter);
        map.insert(String::from("edit"), Token::Edit);
        map.insert(String::from("e"), Token::Edit);
        map.insert(String::from("clear"), Token::Clear);
        map.insert(String::from("c"), Token::Clear);
        map.insert(String::from("update"), Token::Update);
        map.insert(String::from("u"), Token::Update);
        map.insert(String::from("run"), Token::Run);
        map.insert(String::from("r"), Token::Run);
        map.insert(String::from("show"), Token::Show);
        map.insert(String::from("s"), Token::Show);
        map.insert(String::from("help"), Token::Help);
        map.insert(String::from("h"), Token::Help);
        Scanner {
            keywords: map,
            current: String::from(""),
            tokens: Vec::new(),
            is_sha: false,
        }
    }

    /// Scans the input command to return a stream of tokens.
    pub fn scan(&mut self, cmd: String) -> Vec<Token> {
        for c in cmd.chars() {
            match c {
                '#' => {
                    self.tokenize();
                    self.is_sha = true;
                }
                '*' => {
                    self.tokenize();
                    self.tokens.push(Token::Star);
                }
                _ => {
                    if c.is_whitespace() {
                        self.tokenize();
                    } else {
                        self.current.push(c);
                    }
                }
            }
        }
        self.tokenize();
        self.tokens.push(Token::EOS);
        std::mem::replace(&mut self.tokens, Vec::new())
    }

    /// Convert the current characters to the equivalent token and push it
    /// to the `tokens` queue.
    fn tokenize(&mut self) {
        if self.current.len() > 0 {
            let word = std::mem::replace(&mut self.current, String::new());
            if self.is_sha {
                self.tokens.push(Token::Sha(word));
            } else {
                self.tokens.push(self.to_token(word));
            }
        }
        self.is_sha = false;
    }

    /// Converts a word to the corresponding token.
    fn to_token(&self, token: String) -> Token {
        match self.keywords.get(&token) {
            Some(keyword) => keyword.clone(),
            None => Token::Lit(token),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Star => write!(f, "*"),
            Token::Run => write!(f, "run"),
            Token::EOS => write!(f, "VOID"),
            Token::Edit => write!(f, "edit"),
            Token::Show => write!(f, "show"),
            Token::Quit => write!(f, "quit"),
            Token::Help => write!(f, "help"),
            Token::Clear => write!(f, "clear"),
            Token::Lit(s) => write!(f, "{}", s),
            Token::Sha(t) => write!(f, "#{}", t),
            Token::Update => write!(f, "update"),
            Token::Filter => write!(f, "filter"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan() {
        let mut scanner = Scanner::new();
        assert_eq!(
            vec![Token::Show, Token::Star, Token::EOS],
            scanner.scan(String::from("show    *"))
        );
        assert_eq!(
            vec![Token::Quit, Token::Quit, Token::EOS],
            scanner.scan(String::from("q quit"))
        );
        assert_eq!(
            vec![Token::Filter, Token::Sha(String::from("test")), Token::EOS],
            scanner.scan(String::from("f #test"))
        );
    }
}
