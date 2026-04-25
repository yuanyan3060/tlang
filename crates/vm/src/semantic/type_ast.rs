use ast::{BinaryOp, UnaryOp};
use token::Literal;

use crate::semantic::{scope::Location, ty::TypeId};

#[derive(Debug, Clone)]
pub struct Program {
    pub defs: Vec<Definition>,
}

#[derive(Debug, Clone)]
pub enum Definition {
    StructDef(StructDef),
    FunctionDef(FunctionDef),
    ImplDef(ImplDef),
}

#[derive(Debug, Clone)]
pub struct ImplDef {
    pub ty: TypeId,
    pub functions: Vec<AssociatedFunction>,
}

#[derive(Debug, Clone)]
pub enum AssociatedFunction {
    Function(FunctionDef),
    Method(FunctionDef),
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_: TypeId,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<Arg>,
    pub return_type: Option<TypeId>,
    pub local_count: usize,
    pub body: Block,
    pub idx: usize
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub type_: TypeId,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<BlockStmt>,
}

#[derive(Debug, Clone)]
pub enum BlockStmt {
    Let(LetStmt),
    Assign(AssignStmt),
    Return(ReturnStmt),
    Expr(Expr),
    Block(Box<Block>),
    If(IfStmt),
    While(WhileStmt),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub var_name: String,
    pub location: Location,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub target: Expr,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Box<Expr>,
    pub then_branch: Box<Block>,
    pub else_branch: Option<Box<Block>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Box<Expr>,
    pub block: Box<Block>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal {
        value: Literal,
        ty: TypeId
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        ty: TypeId
    },

    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        ty: TypeId
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        ty: TypeId
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        ty: TypeId
    },
    Member {
        target: Box<Expr>,
        member: String,
        offset: usize,
        member_ty: TypeId
    },
    Struct {
        struct_ty: TypeId,
        fields: Vec<FieldAssign>,
    },
    Path {
        segments: Vec<PathSegment>,
        location: Location,
        ty: TypeId
    },
    Method {
        this_ty: TypeId,
        method_name: String,
        method_ty: TypeId,
        location: Location,
    },
}

impl Expr {
    pub fn ty(&self) -> &TypeId {
        match self {
            Expr::Literal { ty, ..  } => ty,
            Expr::Unary { ty, ..  } => ty,
            Expr::Binary { ty, ..  } => ty,
            Expr::Call { ty, ..  } => ty,
            Expr::Index { ty, ..  } => ty,
            Expr::Member{ member_ty: ty, ..  } => ty,
            Expr::Struct { struct_ty, ..  } => struct_ty,
            Expr::Path { ty, ..  } => ty,
            Expr::Method { method_ty, .. } => method_ty
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PathSegment {
    pub ident: String,
    pub args: Vec<TypeId>,
}

#[derive(Debug, Clone)]
pub struct FieldAssign {
    pub name: String,
    pub offset: usize,
    pub expr: Expr,
}
