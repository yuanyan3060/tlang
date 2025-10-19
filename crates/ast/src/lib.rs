use token::Literal;

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    StructDef(StructDef),
    FunctionDef(FunctionDef),
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub functions: Vec<FunctionDef>,
    pub methods: Vec<FunctionDef>,
}

#[derive(Debug)]
pub struct Type {
    pub name: String,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<Arg>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug)]
pub struct Arg {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<BlockStmt>,
}

#[derive(Debug)]
pub enum BlockStmt {
    Let(LetStmt),
    Assign(AssignStmt),
    Return(ReturnStmt),
    Expr(Expr),
    Block(Box<Block>),
}

#[derive(Debug)]
pub struct LetStmt {
    pub var_name: String,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct AssignStmt {
    pub target: Expr,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Ident(String),
    Literal(Literal),

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Member {
        target: Box<Expr>,
        member: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Plus,   // +x
    Minus,  // -x
    Not,    // !x
    BitNot, // ^x
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %

    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    ShiftLeft,  // <<
    ShiftRight, // >>

    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    And, // &&
    Or,  // ||
}
