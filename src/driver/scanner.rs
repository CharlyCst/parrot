use std::collections::HashMap;

pub struct ParseError {
    pub message: String,
    pos: usize,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Quit,
    Filter,
    Clear,
    Run,
    Show,
    Star,
    Sha,
    Lit(String),
}

pub struct Scanner {
    keywords: HashMap<String, Token>,
    current: String,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new() -> Scanner {
        let mut map = HashMap::new();
        map.insert(String::from("quit"), Token::Quit);
        map.insert(String::from("q"), Token::Quit);
        map.insert(String::from("filter"), Token::Filter);
        map.insert(String::from("f"), Token::Filter);
        map.insert(String::from("clear"), Token::Clear);
        map.insert(String::from("c"), Token::Clear);
        map.insert(String::from("run"), Token::Run);
        map.insert(String::from("r"), Token::Run);
        map.insert(String::from("show"), Token::Show);
        map.insert(String::from("s"), Token::Show);

        Scanner {
            keywords: map,
            current: String::from(""),
            tokens: Vec::new(),
        }
    }

    /// Scans the input command to return a stream of tokens.
    pub fn scan(&mut self, cmd: String) -> Vec<Token> {
        for c in cmd.chars() {
            match c {
                '#' => {
                    self.tokenize();
                    self.tokens.push(Token::Sha);
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
        std::mem::replace(&mut self.tokens, Vec::new())
    }

    /// Convert the current characters to the equivalent token and push it 
    /// to the `tokens` queue.
    fn tokenize(&mut self) {
        if self.current.len() > 0 {
            let word = std::mem::replace(&mut self.current, String::new());
            self.tokens.push(self.to_token(word));
        }
    }

    /// Converts a word to the corresponding token.
    fn to_token(&self, token: String) -> Token {
        match self.keywords.get(&token) {
            Some(keyword) => keyword.clone(),
            None => Token::Lit(token),
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
            vec![Token::Show, Token::Star],
            scanner.scan(String::from("show    *"))
        );
        assert_eq!(
            vec![Token::Quit, Token::Quit],
            scanner.scan(String::from("q quit"))
        );
        assert_eq!(
            vec![Token::Filter, Token::Sha, Token::Lit(String::from("test"))],
            scanner.scan(String::from("f #test"))
        );
    }
}
