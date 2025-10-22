use std::fmt::{self, Debug};

#[derive(PartialEq, Clone)]
pub enum Token {
    Whitespace,
    /// =
    Assign,
    /// ->
    Arrow,
    /// .
    Dot,
    /// ,
    Comma,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// [
    OpenBracket,
    /// ]
    CloseBracket,

    /// ==
    Equal,
    /// !=
    NotEqual,
    /// <
    Less,
    /// <=
    LessEqual,
    /// >
    Greater,
    /// >=
    GreaterEqual,

    /// &&
    And,
    /// ||
    Or,
    /// !
    Not,

    /// &
    BitAnd,
    /// |
    BitOr,
    /// ^
    BitXor,
    /// ~
    BitNot,
    /// <<
    ShiftLeft,
    /// >>
    ShiftRight,

    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,

    Ident(String),
    Literal(Literal),

    Let,
    Fn,
    Struct,
    Return,
    SelfArg,
    If,
    While,
    Else,
    Break,
    Continue,
    // ::
    Path,

    /// \n
    NewLine,
    Unknown,
    Eof,
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Whitespace => TokenKind::Whitespace,
            Token::Assign => TokenKind::Assign,
            Token::Arrow => TokenKind::Arrow,
            Token::Dot => TokenKind::Dot,
            Token::Comma => TokenKind::Comma,
            Token::Colon => TokenKind::Colon,
            Token::Semicolon => TokenKind::Semicolon,
            Token::OpenParen => TokenKind::OpenParen,
            Token::CloseParen => TokenKind::CloseParen,
            Token::OpenBrace => TokenKind::OpenBrace,
            Token::CloseBrace => TokenKind::CloseBrace,
            Token::OpenBracket => TokenKind::OpenBracket,
            Token::CloseBracket => TokenKind::CloseBracket,
            Token::Equal => TokenKind::Equal,
            Token::NotEqual => TokenKind::NotEqual,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessEqual,
            Token::Greater => TokenKind::Greater,
            Token::GreaterEqual => TokenKind::GreaterEqual,
            Token::And => TokenKind::And,
            Token::Or => TokenKind::Or,
            Token::Not => TokenKind::Not,
            Token::BitAnd => TokenKind::BitAnd,
            Token::BitOr => TokenKind::BitOr,
            Token::BitXor => TokenKind::BitXor,
            Token::BitNot => TokenKind::BitNot,
            Token::ShiftLeft => TokenKind::ShiftLeft,
            Token::ShiftRight => TokenKind::ShiftRight,
            Token::Plus => TokenKind::Plus,
            Token::Minus => TokenKind::Minus,
            Token::Star => TokenKind::Star,
            Token::Slash => TokenKind::Slash,
            Token::Percent => TokenKind::Percent,
            Token::Ident(_) => TokenKind::Ident,
            Token::Literal(_) => TokenKind::Literal,
            Token::Let => TokenKind::Let,
            Token::Fn => TokenKind::Fn,
            Token::Struct => TokenKind::Struct,
            Token::Return => TokenKind::Return,
            Token::SelfArg => TokenKind::SelfArg,
            Token::If => TokenKind::If,
            Token::While => TokenKind::While,
            Token::Else => TokenKind::Else,
            Token::Break => TokenKind::Break,
            Token::Continue => TokenKind::Continue,
            Token::Path => TokenKind::Path,
            Token::NewLine => TokenKind::NewLine,
            Token::Unknown => TokenKind::Unknown,
            Token::Eof => TokenKind::Eof,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Whitespace => write!(f, "whitespace"),
            Token::Assign => write!(f, "="),
            Token::Arrow => write!(f, "->"),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::BitAnd => write!(f, "&"),
            Token::BitOr => write!(f, "|"),
            Token::BitXor => write!(f, "^"),
            Token::BitNot => write!(f, "~"),
            Token::ShiftLeft => write!(f, "<<"),
            Token::ShiftRight => write!(f, ">>"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Ident(ident) => write!(f, "ident({})", ident),
            Token::Literal(literal) => write!(f, "literal({:?})", literal),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            Token::Struct => write!(f, "struct"),
            Token::Return => write!(f, "return"),
            Token::SelfArg => write!(f, "self"),
            Token::If => write!(f, "if"),
            Token::While => write!(f, "while"),
            Token::Else => write!(f, "else"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Path => write!(f, "::"),
            Token::NewLine => write!(f, "\\n"),
            Token::Unknown => write!(f, "unknown"),
            Token::Eof => write!(f, "eof"),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Literal {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::Bool(v) => write!(f, "{}", v),
            Literal::Int(v) => write!(f, "{}", v),
            Literal::Float(v) => write!(f, "{}", v),
            Literal::String(text) => write!(f, "\"{}\"", text),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenKind {
    Whitespace,
    /// =
    Assign,
    /// ->
    Arrow,
    /// .
    Dot,
    /// ,
    Comma,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// [
    OpenBracket,
    /// ]
    CloseBracket,

    /// ==
    Equal,
    /// !=
    NotEqual,
    /// <
    Less,
    /// <=
    LessEqual,
    /// >
    Greater,
    /// >=
    GreaterEqual,

    /// &&
    And,
    /// ||
    Or,
    /// !
    Not,

    /// &
    BitAnd,
    /// |
    BitOr,
    /// ^
    BitXor,
    /// ~
    BitNot,
    /// <<
    ShiftLeft,
    /// >>
    ShiftRight,

    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,

    Ident,
    Literal,

    Let,
    Fn,
    Struct,
    Return,
    SelfArg,
    If,
    While,
    Else,
    Break,
    Continue,
    Path,

    /// \n
    NewLine,
    Unknown,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::OpenParen => write!(f, "("),
            TokenKind::CloseParen => write!(f, ")"),
            TokenKind::OpenBrace => write!(f, "{{"),
            TokenKind::CloseBrace => write!(f, "}}"),
            TokenKind::OpenBracket => write!(f, "["),
            TokenKind::CloseBracket => write!(f, "]"),
            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::BitAnd => write!(f, "&"),
            TokenKind::BitOr => write!(f, "|"),
            TokenKind::BitXor => write!(f, "^"),
            TokenKind::BitNot => write!(f, "~"),
            TokenKind::ShiftLeft => write!(f, "<<"),
            TokenKind::ShiftRight => write!(f, ">>"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Ident => write!(f, "ident"),
            TokenKind::Literal => write!(f, "literal"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::SelfArg => write!(f, "self"),
            TokenKind::If => write!(f, "if"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Path => write!(f, "::"),
            TokenKind::NewLine => write!(f, "\\n"),
            TokenKind::Unknown => write!(f, "unknown"),
            TokenKind::Eof => write!(f, "eof"),
        }
    }
}
