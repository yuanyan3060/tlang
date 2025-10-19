use std::fmt::Display;

use lex::Pos;
use token::{Token, TokenKind};

#[derive(Debug)]
pub enum ParseError {
    Unexpected {
        expected: Vec<TokenKind>,
        found: Token,
        pos: Pos,
    },
    SelfArgNotFirst {
        pos: Pos,
    },
    SelfArgInFunction {
        pos: Pos,
    },
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
