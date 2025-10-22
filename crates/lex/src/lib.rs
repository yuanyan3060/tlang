use std::str::Chars;

use token::{Literal, Token};
use unicode_xid::UnicodeXID;

#[derive(Clone, Copy, Default, Debug)]
pub struct Pos {
    line: usize,
    offset: usize,
}

pub const EOF_CHAR: char = '\0';

#[derive(Clone)]
pub struct Lex<'a> {
    input: Chars<'a>,
    pos: Pos,
}

impl<'a> Lex<'a> {
    pub fn new(input: Chars<'a>) -> Self {
        Self {
            input,
            pos: Pos::default(),
        }
    }

    pub fn bump(&mut self) -> Option<char> {
        match self.input.next() {
            Some(value) => {
                self.pos.offset += 1;
                Some(value)
            }
            None => None,
        }
    }

    pub fn first(&self) -> char {
        self.input.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn second(&self) -> char {
        self.input.clone().nth(1).unwrap_or(EOF_CHAR)
    }

    pub fn peek_check(&self, text: &str) -> bool {
        self.input.as_str().starts_with(text)
    }

    pub fn skip(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.pos.offset += 1;
        self.input.nth(n - 1);
    }

    pub fn all(mut self) -> Vec<(Token, Pos)> {
        let mut tokens = Vec::new();
        loop {
            let (token, pos) = self.advance_token();

            if token == Token::Whitespace {
                continue;
            }

            if token == Token::Eof {
                tokens.push((token, pos));
                break;
            }
            tokens.push((token, pos));
        }
        tokens
    }

    pub fn advance_token(&mut self) -> (Token, Pos) {
        let pos = self.pos;
        let Some(first_char) = self.bump() else {
            return (Token::Eof, pos);
        };

        let token = match first_char {
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '{' => Token::OpenBrace,
            '}' => Token::CloseBrace,
            '[' => Token::OpenBracket,
            ']' => Token::CloseBracket,
            '.' => Token::Dot,
            ',' => Token::Comma,
            ':' => {
                if self.peek_check(":") {
                    self.bump();
                    Token::Path
                } else {
                    Token::Colon
                }
            }
            ';' => Token::Semicolon,
            '=' => {
                if self.peek_check("=") {
                    self.bump();
                    Token::Equal
                } else {
                    Token::Assign
                }
            }
            '!' => {
                if self.peek_check("=") {
                    self.bump();
                    Token::NotEqual
                } else {
                    Token::Not
                }
            }
            '<' => {
                if self.peek_check("<") {
                    self.bump();
                    Token::ShiftLeft
                } else if self.peek_check("=") {
                    self.bump();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.peek_check(">") {
                    self.bump();
                    Token::ShiftRight
                } else if self.peek_check("=") {
                    self.bump();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '~' => Token::BitNot,
            '&' => {
                if self.peek_check("&") {
                    self.bump();
                    Token::And
                } else {
                    Token::BitAnd
                }
            }
            '|' => {
                if self.peek_check("|") {
                    self.bump();
                    Token::Or
                } else {
                    Token::BitOr
                }
            }
            '^' => Token::BitXor,
            '"' => {
                let mut text = "".to_string();
                while let Some(c) = self.bump() {
                    match c {
                        '"' => break,
                        '\\' if self.first() == '\\' || self.first() == '"' => {
                            text.push(c);
                            text.push(self.bump().unwrap())
                        }
                        _ => text.push(c),
                    }
                }
                let text = unescape::unescape(&text).expect("unescape failed");
                Token::Literal(token::Literal::String(text))
            }
            '+' => Token::Plus,
            '-' => {
                if self.peek_check(">") {
                    self.bump();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            c if c.is_numeric() => {
                let mut is_float = false;
                let mut text = c.to_string();
                loop {
                    let c = self.first();
                    match c {
                        c if c.is_numeric() => {
                            self.bump();
                            text.push(c);
                        }
                        '.' => {
                            if !self.second().is_numeric() {
                                break;
                            }
                            self.bump();
                            is_float = true;
                            text.push(c);
                        }
                        _ => break,
                    }
                }

                if is_float {
                    let val = text.parse().expect("valid f64");
                    Token::Literal(token::Literal::Float(val))
                } else {
                    let val = text.parse().expect("valid int64");
                    Token::Literal(token::Literal::Int(val))
                }
            }
            '\n' => {
                self.pos.offset = 0;
                self.pos.line += 1;
                Token::NewLine
            }
            '\r' if self.peek_check("\n") => {
                self.skip(1);
                self.pos.offset = 0;
                self.pos.line += 1;
                Token::NewLine
            }
            c if c.is_xid_start() => {
                let mut ident = c.to_string();
                while self.first().is_xid_continue() {
                    ident.push(self.bump().unwrap());
                }
                match ident.as_str() {
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "return" => Token::Return,
                    "struct" => Token::Struct,
                    "self" => Token::SelfArg,
                    "nil" => Token::Literal(Literal::Nil),
                    "true" => Token::Literal(Literal::Bool(true)),
                    "false" => Token::Literal(Literal::Bool(false)),
                    "if" => Token::If,
                    "while" => Token::While,
                    "else" => Token::Else,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    _ => Token::Ident(ident),
                }
            }
            c if c.is_whitespace() => Token::Whitespace,
            _ => Token::Unknown,
        };
        (token, pos)
    }
}

pub fn pretty_print(tokens: &[(Token, Pos)]) {
    let mut line = 0;
    print!("{}: ", line + 1);
    for (token, pos) in tokens {
        if pos.line != line {
            line = pos.line;
            print!("\n{}: ", line + 1);
        }
        print!("{:?} ", token)
    }
    println!()
}
