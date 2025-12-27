//! Lexer for MicroPerl

use crate::token::{Token, TokenWithSpan};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    last_token: Option<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            last_token: None,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '#' {
                // Skip comment to end of line
                while let Some(c) = self.current() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                if let Some(next) = self.peek() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if c == '_' {
                // Perl allows underscores in numbers for readability
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(num_str.parse().unwrap_or(0.0))
        } else {
            Token::Integer(num_str.parse().unwrap_or(0))
        }
    }

    fn read_string(&mut self, quote: char) -> Token {
        self.advance(); // consume opening quote
        let mut s = String::new();
        let interpolate = quote == '"';

        while let Some(c) = self.current() {
            if c == quote {
                self.advance();
                break;
            } else if c == '\\' {
                self.advance();
                if let Some(escaped) = self.current() {
                    let ch = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        '$' => '$',
                        '@' => '@',
                        '0' => '\0',
                        _ => escaped,
                    };
                    s.push(ch);
                    self.advance();
                }
            } else {
                s.push(c);
                self.advance();
            }
        }

        Token::String(s)
    }

    fn read_regex(&mut self) -> Token {
        self.advance(); // consume opening /
        let mut pattern = String::new();

        while let Some(c) = self.current() {
            if c == '/' {
                self.advance();
                break;
            } else if c == '\\' {
                pattern.push(c);
                self.advance();
                if let Some(escaped) = self.current() {
                    pattern.push(escaped);
                    self.advance();
                }
            } else {
                pattern.push(c);
                self.advance();
            }
        }

        // Read flags
        let mut flags = String::new();
        while let Some(c) = self.current() {
            if c.is_alphabetic() {
                flags.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Token::Regex(pattern, flags)
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn read_variable(&mut self, sigil: char) -> Token {
        self.advance(); // consume sigil
        let name = self.read_ident();
        match sigil {
            '$' => Token::ScalarVar(name),
            '@' => Token::ArrayVar(name),
            '%' => Token::HashVar(name),
            _ => unreachable!(),
        }
    }

    pub fn next_token(&mut self) -> TokenWithSpan {
        self.skip_whitespace();

        let line = self.line;
        let column = self.column;

        let token = match self.current() {
            None => Token::Eof,
            Some(c) => match c {
                // Variables - but check if followed by identifier char
                '$' => self.read_variable('$'),
                '@' => {
                    // Check if this is array variable or just @ sigil
                    if let Some(next) = self.peek() {
                        if next.is_alphabetic() || next == '_' {
                            self.read_variable('@')
                        } else {
                            self.advance();
                            Token::At
                        }
                    } else {
                        self.advance();
                        Token::At
                    }
                }
                '%' => {
                    // Check if this is hash variable or modulo operator
                    if let Some(next) = self.peek() {
                        if next.is_alphabetic() || next == '_' {
                            self.read_variable('%')
                        } else if next == '=' {
                            // %= operator
                            self.advance();
                            self.advance();
                            Token::PercentEquals
                        } else {
                            self.advance();
                            Token::Percent
                        }
                    } else {
                        self.advance();
                        Token::Percent
                    }
                }

                // Strings
                '"' | '\'' => self.read_string(c),

                // Numbers
                '0'..='9' => self.read_number(),

                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = self.read_ident();
                    Token::is_keyword(&ident).unwrap_or(Token::Ident(ident))
                }

                // Operators
                '+' => {
                    self.advance();
                    match self.current() {
                        Some('+') => { self.advance(); Token::Increment }
                        Some('=') => { self.advance(); Token::PlusEquals }
                        _ => Token::Plus,
                    }
                }
                '-' => {
                    self.advance();
                    match self.current() {
                        Some('-') => { self.advance(); Token::Decrement }
                        Some('=') => { self.advance(); Token::MinusEquals }
                        Some('>') => { self.advance(); Token::Arrow }
                        _ => Token::Minus,
                    }
                }
                '*' => {
                    self.advance();
                    match self.current() {
                        Some('*') => { self.advance(); Token::DoubleStar }
                        Some('=') => { self.advance(); Token::StarEquals }
                        _ => Token::Star,
                    }
                }
                '/' => {
                    // Check if this is a regex (after =~ or !~)
                    if matches!(self.last_token, Some(Token::Match) | Some(Token::NotMatch)) {
                        self.read_regex()
                    } else {
                        self.advance();
                        match self.current() {
                            Some('=') => { self.advance(); Token::SlashEquals }
                            _ => Token::Slash,
                        }
                    }
                }
                '.' => {
                    self.advance();
                    match self.current() {
                        Some('.') => {
                            self.advance();
                            match self.current() {
                                Some('.') => { self.advance(); Token::Ellipsis }
                                _ => Token::Range,
                            }
                        }
                        Some('=') => { self.advance(); Token::DotEquals }
                        _ => Token::Dot,
                    }
                }

                // Comparison
                '=' => {
                    self.advance();
                    match self.current() {
                        Some('=') => { self.advance(); Token::Eq }
                        Some('~') => { self.advance(); Token::Match }
                        Some('>') => { self.advance(); Token::FatArrow }
                        _ => Token::Assign,
                    }
                }
                '!' => {
                    self.advance();
                    match self.current() {
                        Some('=') => { self.advance(); Token::Ne }
                        Some('~') => { self.advance(); Token::NotMatch }
                        _ => Token::Not,
                    }
                }
                '<' => {
                    self.advance();
                    match self.current() {
                        Some('=') => {
                            self.advance();
                            match self.current() {
                                Some('>') => { self.advance(); Token::Cmp }
                                _ => Token::Le,
                            }
                        }
                        Some('<') => { self.advance(); Token::ShiftLeft }
                        _ => Token::Lt,
                    }
                }
                '>' => {
                    self.advance();
                    match self.current() {
                        Some('=') => { self.advance(); Token::Ge }
                        Some('>') => { self.advance(); Token::ShiftRight }
                        _ => Token::Gt,
                    }
                }

                // Logical
                '&' => {
                    self.advance();
                    match self.current() {
                        Some('&') => {
                            self.advance();
                            match self.current() {
                                Some('=') => { self.advance(); Token::AndEquals }
                                _ => Token::And,
                            }
                        }
                        _ => Token::BitAnd,
                    }
                }
                '|' => {
                    self.advance();
                    match self.current() {
                        Some('|') => {
                            self.advance();
                            match self.current() {
                                Some('=') => { self.advance(); Token::OrEquals }
                                _ => Token::Or,
                            }
                        }
                        _ => Token::BitOr,
                    }
                }
                '^' => { self.advance(); Token::BitXor }
                '~' => { self.advance(); Token::BitNot }

                // Delimiters
                '(' => { self.advance(); Token::LParen }
                ')' => { self.advance(); Token::RParen }
                '[' => { self.advance(); Token::LBracket }
                ']' => { self.advance(); Token::RBracket }
                '{' => { self.advance(); Token::LBrace }
                '}' => { self.advance(); Token::RBrace }
                ';' => { self.advance(); Token::Semicolon }
                ',' => { self.advance(); Token::Comma }
                ':' => {
                    self.advance();
                    match self.current() {
                        Some(':') => { self.advance(); Token::DoubleColon }
                        _ => Token::Colon,
                    }
                }
                '\\' => { self.advance(); Token::Backslash }
                '?' => { self.advance(); Token::Question }

                _ => {
                    self.advance();
                    Token::Eof // Unknown character, skip
                }
            }
        };

        self.last_token = Some(token.clone());
        TokenWithSpan { token, line, column }
    }

    pub fn tokenize(&mut self) -> Vec<TokenWithSpan> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.token == Token::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variables() {
        let mut lexer = Lexer::new("$x @arr %hash");
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "x"));
        assert!(matches!(lexer.next_token().token, Token::ArrayVar(s) if s == "arr"));
        assert!(matches!(lexer.next_token().token, Token::HashVar(s) if s == "hash"));
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new("\"hello world\"");
        assert!(matches!(lexer.next_token().token, Token::String(s) if s == "hello world"));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("my if while sub");
        assert!(matches!(lexer.next_token().token, Token::My));
        assert!(matches!(lexer.next_token().token, Token::If));
        assert!(matches!(lexer.next_token().token, Token::While));
        assert!(matches!(lexer.next_token().token, Token::Sub));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != =~ ..");
        assert!(matches!(lexer.next_token().token, Token::Plus));
        assert!(matches!(lexer.next_token().token, Token::Minus));
        assert!(matches!(lexer.next_token().token, Token::Star));
        assert!(matches!(lexer.next_token().token, Token::Slash));
        assert!(matches!(lexer.next_token().token, Token::Eq));
        assert!(matches!(lexer.next_token().token, Token::Ne));
        assert!(matches!(lexer.next_token().token, Token::Match));
        assert!(matches!(lexer.next_token().token, Token::Range));
    }

    // === Regex lexer tests ===

    #[test]
    fn test_regex_after_match_operator() {
        let mut lexer = Lexer::new("$x =~ /hello/");
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "x"));
        assert!(matches!(lexer.next_token().token, Token::Match));
        assert!(matches!(lexer.next_token().token, Token::Regex(p, f) if p == "hello" && f.is_empty()));
    }

    #[test]
    fn test_regex_after_not_match_operator() {
        let mut lexer = Lexer::new("$x !~ /world/");
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "x"));
        assert!(matches!(lexer.next_token().token, Token::NotMatch));
        assert!(matches!(lexer.next_token().token, Token::Regex(p, f) if p == "world" && f.is_empty()));
    }

    #[test]
    fn test_regex_with_flags() {
        let mut lexer = Lexer::new("$x =~ /pattern/gi");
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, f) if p == "pattern" && f == "gi"));
    }

    #[test]
    fn test_regex_empty_pattern() {
        let mut lexer = Lexer::new("$x =~ //");
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p.is_empty()));
    }

    #[test]
    fn test_regex_with_wildcard() {
        let mut lexer = Lexer::new("$x =~ /h.llo/");
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == "h.llo"));
    }

    #[test]
    fn test_regex_with_escaped_slash() {
        let mut lexer = Lexer::new(r#"$x =~ /path\/to\/file/"#);
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == r"path\/to\/file"));
    }

    #[test]
    fn test_regex_with_special_chars() {
        let mut lexer = Lexer::new(r#"$x =~ /\d+\s*/"#);
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == r"\d+\s*"));
    }

    #[test]
    fn test_slash_as_division_not_regex() {
        // Without =~ or !~ preceding, / should be division
        let mut lexer = Lexer::new("$x / $y");
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "x"));
        assert!(matches!(lexer.next_token().token, Token::Slash));
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "y"));
    }

    #[test]
    fn test_slash_equals_not_regex() {
        let mut lexer = Lexer::new("$x /= 2");
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "x"));
        assert!(matches!(lexer.next_token().token, Token::SlashEquals));
        assert!(matches!(lexer.next_token().token, Token::Integer(2)));
    }

    #[test]
    fn test_regex_in_condition() {
        let mut lexer = Lexer::new("if ($x =~ /test/) { }");
        assert!(matches!(lexer.next_token().token, Token::If));
        assert!(matches!(lexer.next_token().token, Token::LParen));
        assert!(matches!(lexer.next_token().token, Token::ScalarVar(s) if s == "x"));
        assert!(matches!(lexer.next_token().token, Token::Match));
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == "test"));
        assert!(matches!(lexer.next_token().token, Token::RParen));
    }

    #[test]
    fn test_multiple_regex_in_sequence() {
        let mut lexer = Lexer::new("$a =~ /one/ && $b !~ /two/");
        lexer.next_token(); // $a
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == "one"));
        assert!(matches!(lexer.next_token().token, Token::And));
        lexer.next_token(); // $b
        lexer.next_token(); // !~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == "two"));
    }

    #[test]
    fn test_regex_with_anchor_chars() {
        let mut lexer = Lexer::new("$x =~ /^start.*end$/");
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == "^start.*end$"));
    }

    #[test]
    fn test_regex_with_character_class() {
        let mut lexer = Lexer::new("$x =~ /[a-z]+/");
        lexer.next_token(); // $x
        lexer.next_token(); // =~
        assert!(matches!(lexer.next_token().token, Token::Regex(p, _) if p == "[a-z]+"));
    }
}
