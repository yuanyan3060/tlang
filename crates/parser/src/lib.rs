use ast::{FunctionDef, IfStmt, LetStmt, ReturnStmt, WhileStmt};
use lex::Pos;

use std::slice::Iter;
use token::{Token, TokenKind};

use crate::error::{ParseError, ParseResult};

pub mod error;

pub struct Parser<'a> {
    input: Iter<'a, (Token, Pos)>,
    eof_pos: Pos,
}

impl<'a> Parser<'a> {
    pub fn new(input: Iter<'a, (Token, Pos)>) -> Self {
        let eof_pos = match input.as_slice().last() {
            Some((Token::Eof, pos)) => *pos,
            Some((token, _)) => {
                println!("{:?}", token);
                panic!("last token must be eof")
            }
            None => Pos::default(),
        };

        Self { input, eof_pos }
    }

    pub fn bump(&mut self) -> (&Token, Pos) {
        if let Some((token, pos)) = self.input.next() {
            (token, *pos)
        } else {
            (&Token::Eof, self.eof_pos)
        }
    }

    pub fn first(&self) -> &Token {
        self.input
            .clone()
            .next()
            .map(|x| &x.0)
            .unwrap_or(&Token::Eof)
    }

    pub fn second(&self) -> &Token {
        self.input
            .clone()
            .nth(1)
            .map(|x| &x.0)
            .unwrap_or(&Token::Eof)
    }

    pub fn first_full(&self) -> (&Token, Pos) {
        if let Some((token, pos)) = self.input.clone().next() {
            (token, *pos)
        } else {
            (&Token::Eof, self.eof_pos)
        }
    }

    pub fn unexpected_eof(&self, kinds: Vec<TokenKind>) -> ParseError {
        ParseError::Unexpected {
            expected: kinds,
            found: Token::Eof,
            pos: self.eof_pos,
        }
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        let (token, pos) = self.first_full();
        if token.kind() == kind {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::Unexpected {
                expected: vec![kind],
                found: token.clone(),
                pos,
            })
        }
    }

    pub fn first_check(&self, kind: TokenKind) -> bool {
        self.input
            .clone()
            .next()
            .map(|(token, _)| token.kind() == kind)
            .unwrap_or(false)
    }

    pub fn skip_newline(&mut self) {
        loop {
            let first = self.first();
            match first {
                Token::Whitespace | Token::NewLine => {
                    self.bump();
                }
                _ => break,
            }
        }
    }

    pub fn expect_ident(&mut self) -> Result<String, ParseError> {
        let (token, pos) = self.bump();
        match token {
            Token::Ident(name) => Ok(name.to_string()),
            _ => Err(ParseError::Unexpected {
                expected: vec![TokenKind::Ident],
                found: token.clone(),
                pos,
            }),
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<ast::Program> {
        let mut program = ast::Program {
            statements: Vec::new(),
        };

        loop {
            self.skip_newline();
            match self.first_full() {
                (Token::Struct, _) => {
                    let node = self.parse_struct()?;
                    program.statements.push(ast::Statement::StructDef(node));
                }
                (Token::Fn, _) => {
                    let function = self.parse_fn()?;
                    program
                        .statements
                        .push(ast::Statement::FunctionDef(function));
                }
                (Token::Eof, _) => {
                    break;
                }
                (token, pos) => {
                    return Err(ParseError::Unexpected {
                        expected: vec![TokenKind::Struct, TokenKind::Fn],
                        found: token.clone(),
                        pos,
                    });
                }
            }
        }

        Ok(program)
    }

    pub fn parse_struct(&mut self) -> ParseResult<ast::StructDef> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut functions = Vec::new();

        self.skip_newline();
        self.expect(TokenKind::Struct)?;
        self.skip_newline();

        let name = self.expect_ident()?;

        self.skip_newline();
        self.expect(TokenKind::OpenBrace)?;

        loop {
            self.skip_newline();
            match self.first_full() {
                (Token::Fn, _) => match self.parse_fn_and_method(true)? {
                    RawFunction::Function(function) => {
                        functions.push(function);
                    }
                    RawFunction::Method(method) => {
                        methods.push(method);
                    }
                },
                (Token::Ident(..), _) => {
                    let field = self.parse_field()?;
                    fields.push(field);
                }
                (Token::CloseBrace, _) => {
                    self.bump();
                    break;
                }
                (token, pos) => {
                    return Err(ParseError::Unexpected {
                        expected: vec![TokenKind::Fn, TokenKind::Ident, TokenKind::CloseBrace],
                        found: token.clone(),
                        pos,
                    });
                }
            }
        }
        Ok(ast::StructDef {
            name,
            fields,
            functions,
            methods,
        })
    }

    pub fn parse_ty(&mut self) -> ParseResult<ast::Type> {
        let segments = self.parse_path_segments()?;
        Ok(ast::Type { segments })
    }

    pub fn parse_path_segments(&mut self) -> ParseResult<Vec<ast::PathSegment>> {
        self.skip_newline();
        let mut segments = Vec::new();
        let segment = self.parse_path_segment()?;
        segments.push(segment);

        loop {
            self.skip_newline();
            if !self.first_check(TokenKind::Path) {
                break;
            }
            self.bump();

            self.skip_newline();
            if self.first_check(TokenKind::Less) {
                segments.last_mut().unwrap().args = self.parse_type_args()?;
            } else {
                let segment = self.parse_path_segment()?;
                segments.push(segment);
            }
        }

        Ok(segments)
    }

    pub fn parse_path_segment(&mut self) -> ParseResult<ast::PathSegment> {
        self.skip_newline();
        let ident = self.expect_ident()?;
        self.skip_newline();
        let args = if self.first_check(TokenKind::Less) {
            self.parse_type_args()?
        } else {
            Vec::new()
        };

        Ok(ast::PathSegment { ident, args })
    }

    pub fn parse_type_args(&mut self) -> ParseResult<Vec<ast::Type>> {
        self.skip_newline();
        self.expect(TokenKind::Less)?;
        let mut args = Vec::new();
        loop {
            self.skip_newline();
            if self.first_check(TokenKind::Greater) {
                self.bump();
                break;
            }
            let arg = self.parse_ty()?;
            args.push(arg);
            self.skip_newline();

            if self.first_check(TokenKind::Comma) {
                self.bump();
            }
        }

        Ok(args)
    }

    pub fn parse_fn(&mut self) -> ParseResult<ast::FunctionDef> {
        match self.parse_fn_and_method(false)? {
            RawFunction::Function(function) => Ok(function),
            RawFunction::Method(_) => {
                unreachable!()
            }
        }
    }

    pub fn parse_fn_and_method(&mut self, method_ok: bool) -> ParseResult<RawFunction> {
        self.skip_newline();
        self.expect(TokenKind::Fn)?;

        self.skip_newline();
        let name = self.expect_ident()?;
        let mut args = Vec::new();
        let mut return_type = None;
        let body;

        self.skip_newline();
        self.expect(TokenKind::OpenParen)?;

        let mut is_method = false;
        let mut state = ParseArgState::Start;
        loop {
            self.skip_newline();
            match self.first_full() {
                (Token::SelfArg, pos) => {
                    if !state.can_be_arg() {
                        return Err(ParseError::Unexpected {
                            expected: state.expect(),
                            found: Token::SelfArg,
                            pos,
                        });
                    }

                    if !state.can_be_self() {
                        return Err(ParseError::SelfArgNotFirst { pos });
                    }

                    if !method_ok {
                        return Err(ParseError::SelfArgInFunction { pos });
                    }

                    self.bump();
                    is_method = true;
                    state.next_state();
                }
                (Token::Ident(name), pos) => {
                    let name = name.to_string();

                    if !state.can_be_arg() {
                        return Err(ParseError::Unexpected {
                            expected: state.expect(),
                            found: Token::Ident(name),
                            pos,
                        });
                    }
                    self.bump();
                    self.skip_newline();
                    self.expect(TokenKind::Colon)?;

                    self.skip_newline();
                    let type_ = self.parse_ty()?;
                    args.push(ast::Arg {
                        name: name.to_string(),
                        type_,
                    });
                    state.next_state();
                }
                (Token::CloseParen, pos) => {
                    if !state.can_be_close() {
                        return Err(ParseError::Unexpected {
                            expected: state.expect(),
                            found: Token::CloseParen,
                            pos,
                        });
                    }
                    self.bump();
                    break;
                }
                (Token::Comma, pos) => {
                    if !state.can_be_comma() {
                        return Err(ParseError::Unexpected {
                            expected: state.expect(),
                            found: Token::Comma,
                            pos,
                        });
                    }
                    self.bump();
                    state.next_state();
                }
                (token, pos) => {
                    return Err(ParseError::Unexpected {
                        expected: state.expect(),
                        found: token.clone(),
                        pos,
                    });
                }
            }
        }
        self.skip_newline();
        match self.first_full() {
            (Token::Arrow, _) => {
                self.bump();
                self.skip_newline();
                let ty = self.parse_ty()?;
                return_type = Some(ty);
                self.skip_newline();
                body = self.parse_block()?;
            }
            (Token::OpenBrace, _) => {
                body = self.parse_block()?;
            }
            (token, pos) => {
                return Err(ParseError::Unexpected {
                    expected: vec![TokenKind::Arrow, TokenKind::OpenBrace],
                    found: token.clone(),
                    pos,
                });
            }
        }

        if is_method {
            Ok(RawFunction::Method(FunctionDef {
                name,
                args,
                return_type,
                body,
            }))
        } else {
            Ok(RawFunction::Function(FunctionDef {
                name,
                args,
                return_type,
                body,
            }))
        }
    }

    pub fn parse_field(&mut self) -> ParseResult<ast::Field> {
        self.skip_newline();
        let name = self.expect_ident()?;

        self.skip_newline();
        self.expect(TokenKind::Colon)?;

        self.skip_newline();
        let type_ = self.parse_ty()?;

        self.skip_newline();
        if self.first_check(TokenKind::Comma) {
            self.bump();
        }

        Ok(ast::Field { name, type_ })
    }

    pub fn parse_block(&mut self) -> ParseResult<ast::Block> {
        self.skip_newline();
        self.expect(TokenKind::OpenBrace)?;
        let mut stmts = Vec::new();

        loop {
            self.skip_newline();
            let stmt = match self.first_full() {
                (Token::Let, _) => {
                    self.bump();
                    self.skip_newline();
                    let name = self.expect_ident()?;
                    self.skip_newline();
                    self.expect(TokenKind::Assign)?;
                    self.skip_newline();
                    let expr = self.parse_add_sub_expr(true)?;
                    ast::BlockStmt::Let(LetStmt {
                        var_name: name,
                        expr,
                    })
                }
                (Token::Return, _) => {
                    self.bump();
                    self.skip_newline();
                    let expr = if self.first_check(TokenKind::Semicolon) {
                        self.bump();
                        None
                    } else {
                        let expr = self.parse_add_sub_expr(true)?;
                        Some(expr)
                    };

                    ast::BlockStmt::Return(ReturnStmt { expr })
                }
                (Token::CloseBrace, _) => {
                    self.bump();
                    break;
                }
                (Token::OpenBrace, _) => {
                    let block = self.parse_block()?;
                    ast::BlockStmt::Block(Box::new(block))
                }
                (Token::Semicolon, _) => {
                    self.bump();
                    continue;
                }
                (Token::If, _) => {
                    self.bump();

                    self.skip_newline();
                    let condition = self.parse_expr(false)?;

                    self.skip_newline();
                    let then_branch = self.parse_block()?;

                    self.skip_newline();
                    let else_branch = if self.first_check(TokenKind::Else) {
                        self.bump();
                        self.skip_newline();
                        Some(self.parse_block()?)
                    } else {
                        None
                    };

                    ast::BlockStmt::If(IfStmt {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch: else_branch.map(Box::new),
                    })
                }
                (Token::While, _) => {
                    self.bump();

                    self.skip_newline();
                    let condition = self.parse_expr(false)?;

                    self.skip_newline();
                    let block = self.parse_block()?;

                    ast::BlockStmt::While(WhileStmt {
                        condition: Box::new(condition),
                        block: Box::new(block),
                    })
                }
                (Token::Break, _) => {
                    self.bump();
                    self.skip_newline();
                    self.expect(TokenKind::Semicolon)?;
                    ast::BlockStmt::Break
                }
                (Token::Continue, _) => {
                    self.bump();
                    self.skip_newline();
                    self.expect(TokenKind::Semicolon)?;
                    ast::BlockStmt::Continue
                }
                _ => {
                    let expr = self.parse_expr(true)?;
                    self.skip_newline();
                    let stmt = if self.first_check(TokenKind::Assign) {
                        self.bump();
                        self.skip_newline();
                        let assign = ast::AssignStmt {
                            expr: self.parse_expr(true)?,
                            target: expr,
                        };
                        ast::BlockStmt::Assign(assign)
                    } else {
                        ast::BlockStmt::Expr(expr)
                    };
                    self.expect(TokenKind::Semicolon)?;
                    stmt
                }
            };
            stmts.push(stmt);
        }

        Ok(ast::Block { statements: stmts })
    }

    pub fn parse_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.parse_or_expr(allow_struct)
    }

    pub fn parse_or_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_and_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::Or) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_and_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_and_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_relation_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::And) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_relation_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_relation_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_bit_or_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(
                self.first(),
                Token::Equal
                    | Token::NotEqual
                    | Token::Greater {
                        next_is_greater: false
                    }
                    | Token::GreaterEqual
                    | Token::Less
                    | Token::LessEqual
            ) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_bit_or_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_bit_or_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_bit_xor_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::BitOr) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_bit_xor_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_bit_xor_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_bit_and_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::BitXor) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_bit_and_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_bit_and_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_shift_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::BitAnd) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_shift_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_shift_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_add_sub_expr(allow_struct)?;
        loop {
            self.skip_newline();
            match self.first() {
                Token::ShiftLeft => {
                    let op = self.parse_binary_op()?;
                    self.skip_newline();
                    let right = self.parse_add_sub_expr(allow_struct)?;
                    curr = ast::Expr::Binary {
                        left: Box::new(curr),
                        op,
                        right: Box::new(right),
                    };
                }

                Token::Greater { next_is_greater } => {
                    if !*next_is_greater {
                        break;
                    }

                    self.bump();
                    self.bump();
                    let right = self.parse_add_sub_expr(allow_struct)?;
                    curr = ast::Expr::Binary {
                        left: Box::new(curr),
                        op: ast::BinaryOp::ShiftRight,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(curr)
    }

    pub fn parse_add_sub_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_mul_div_mod_expr(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::Plus | Token::Minus) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_mul_div_mod_expr(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_mul_div_mod_expr(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut curr = self.parse_factor(allow_struct)?;
        loop {
            self.skip_newline();
            if !matches!(self.first(), Token::Star | Token::Slash | Token::Percent) {
                break;
            }
            let op = self.parse_binary_op()?;
            self.skip_newline();
            let right = self.parse_factor(allow_struct)?;
            curr = ast::Expr::Binary {
                left: Box::new(curr),
                op,
                right: Box::new(right),
            };
        }
        Ok(curr)
    }

    pub fn parse_factor(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        match self.first() {
            Token::Not => {
                self.bump();
                Ok(ast::Expr::Unary {
                    op: ast::UnaryOp::Not,
                    expr: Box::new(self.parse_primary(allow_struct)?),
                })
            }
            Token::BitNot => {
                self.bump();
                Ok(ast::Expr::Unary {
                    op: ast::UnaryOp::BitNot,
                    expr: Box::new(self.parse_primary(allow_struct)?),
                })
            }
            Token::Plus => {
                self.bump();
                Ok(ast::Expr::Unary {
                    op: ast::UnaryOp::Plus,
                    expr: Box::new(self.parse_primary(allow_struct)?),
                })
            }
            Token::Minus => {
                self.bump();
                Ok(ast::Expr::Unary {
                    op: ast::UnaryOp::Minus,
                    expr: Box::new(self.parse_primary(allow_struct)?),
                })
            }
            _ => self.parse_primary(allow_struct),
        }
    }

    pub fn parse_primary(&mut self, allow_struct: bool) -> ParseResult<ast::Expr> {
        self.skip_newline();
        let mut expr = match self.first_full() {
            (Token::OpenParen, _) => {
                self.bump();
                let expr = self.parse_or_expr(true)?;
                self.skip_newline();
                self.expect(TokenKind::CloseParen)?;
                expr
            }
            (Token::Ident(_), _) => {
                let segments = self.parse_path_segments()?;
                self.skip_newline();
                if allow_struct && self.first_check(TokenKind::OpenBrace) {
                    self.bump();
                    self.skip_newline();
                    let mut fields = Vec::new();
                    loop {
                        self.skip_newline();
                        if self.first_check(TokenKind::CloseBrace) {
                            self.bump();
                            break;
                        }

                        let field = self.expect_ident()?;
                        self.skip_newline();
                        self.expect(TokenKind::Colon)?;

                        self.skip_newline();
                        let expr = self.parse_expr(true)?;

                        fields.push(ast::FieldAssign { name: field, expr });
                        self.skip_newline();
                        if self.first_check(TokenKind::Comma) {
                            self.bump();
                        }
                    }
                    ast::Expr::Struct {
                        struct_ty: ast::Type { segments },
                        fields,
                    }
                } else {
                    ast::Expr::Path { segments }
                }
            }
            (Token::Literal(literal), _) => {
                let literal = literal.clone();
                self.bump();
                ast::Expr::Literal(literal)
            }
            (Token::SelfArg, _) => {
                self.bump();
                ast::Expr::Path {
                    segments: vec![ast::PathSegment {
                        ident: "self".to_string(),
                        args: Vec::new(),
                    }],
                }
            }
            (token, pos) => {
                return Err(ParseError::Unexpected {
                    expected: vec![],
                    found: token.clone(),
                    pos,
                });
            }
        };

        loop {
            self.skip_newline();
            match self.first_full() {
                (Token::Dot, _) => {
                    self.bump();
                    self.skip_newline();
                    let member = self.expect_ident()?;
                    expr = ast::Expr::Member {
                        target: Box::new(expr),
                        member,
                    };
                }
                (Token::OpenParen, _) => {
                    let args = self.parse_fn_call_args()?;
                    expr = ast::Expr::Call {
                        func: Box::new(expr),
                        args,
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    pub fn parse_fn_call_args(&mut self) -> ParseResult<Vec<ast::Expr>> {
        self.skip_newline();
        self.expect(TokenKind::OpenParen)?;

        let mut comma_ok = false;
        let mut paren_ok = true;

        let mut args = Vec::new();
        loop {
            self.skip_newline();
            match self.first_full() {
                (Token::Comma, pos) => {
                    if !comma_ok {
                        // TODO
                        return Err(ParseError::Unexpected {
                            expected: vec![],
                            found: Token::Comma,
                            pos,
                        });
                    }
                    self.bump();
                    comma_ok = false;
                    paren_ok = false
                }
                (Token::CloseParen, pos) => {
                    if !paren_ok {
                        // TODO
                        return Err(ParseError::Unexpected {
                            expected: vec![],
                            found: Token::CloseParen,
                            pos,
                        });
                    }
                    self.bump();
                    break;
                }
                _ => {
                    args.push(self.parse_expr(true)?);
                    comma_ok = true;
                    paren_ok = true;
                }
            }
        }
        Ok(args)
    }

    pub fn parse_binary_op(&mut self) -> ParseResult<ast::BinaryOp> {
        match self.bump() {
            (Token::Equal, _) => Ok(ast::BinaryOp::Equal),
            (Token::NotEqual, _) => Ok(ast::BinaryOp::NotEqual),
            (Token::Less, _) => Ok(ast::BinaryOp::Less),
            (Token::LessEqual, _) => Ok(ast::BinaryOp::LessEqual),
            (Token::Greater { .. }, _) => Ok(ast::BinaryOp::Greater),
            (Token::GreaterEqual, _) => Ok(ast::BinaryOp::GreaterEqual),

            (Token::BitAnd, _) => Ok(ast::BinaryOp::BitAnd),
            (Token::BitOr, _) => Ok(ast::BinaryOp::BitOr),
            (Token::BitXor, _) => Ok(ast::BinaryOp::BitXor),
            (Token::ShiftLeft, _) => Ok(ast::BinaryOp::ShiftLeft),
            (Token::Plus, _) => Ok(ast::BinaryOp::Add),
            (Token::Minus, _) => Ok(ast::BinaryOp::Subtract),
            (Token::Star, _) => Ok(ast::BinaryOp::Multiply),
            (Token::Slash, _) => Ok(ast::BinaryOp::Divide),
            (Token::Percent, _) => Ok(ast::BinaryOp::Modulo),

            (Token::And, _) => Ok(ast::BinaryOp::And),
            (Token::Or, _) => Ok(ast::BinaryOp::Or),
            (token, pos) => Err(ParseError::Unexpected {
                expected: vec![], // TODO!
                found: token.clone(),
                pos,
            }),
        }
    }
}

pub enum RawFunction {
    Function(ast::FunctionDef),
    Method(ast::FunctionDef),
}

pub enum ParseArgState {
    Start,
    Comma,
    Arg,
}

impl ParseArgState {
    pub fn can_be_self(&self) -> bool {
        matches!(self, Self::Start)
    }

    pub fn can_be_close(&self) -> bool {
        match self {
            Self::Start | Self::Arg => true,
            Self::Comma => false,
        }
    }

    pub fn can_be_arg(&self) -> bool {
        match self {
            Self::Start | Self::Comma => true,
            Self::Arg => false,
        }
    }

    pub fn can_be_comma(&self) -> bool {
        match self {
            Self::Start | Self::Arg => true,
            Self::Comma => false,
        }
    }

    pub fn expect(&self) -> Vec<TokenKind> {
        match self {
            ParseArgState::Start => {
                vec![TokenKind::SelfArg, TokenKind::CloseParen, TokenKind::Ident]
            }
            ParseArgState::Comma => vec![TokenKind::Ident],
            ParseArgState::Arg => vec![TokenKind::Comma, TokenKind::CloseParen],
        }
    }

    pub fn next_state(&mut self) {
        match self {
            ParseArgState::Start => *self = Self::Arg,
            ParseArgState::Comma => *self = Self::Arg,
            ParseArgState::Arg => *self = Self::Comma,
        }
    }
}
