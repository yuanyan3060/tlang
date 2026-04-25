use ast::{BinaryOp, UnaryOp};

use crate::semantic::type_ast;

#[derive(Clone, Copy)]
pub enum Variable {
    Local(usize),
    Global(usize),
    Temp(usize),
}

#[derive(Clone, Copy)]
pub struct Label(usize);

pub enum Operand {
    Variable(Variable),
    ConstNil,
    ConstBool(bool),
    ConstInt(i64),
    ConstFloat(f64),
    ConstString(String),
    Label(Label),
}

pub enum Instruction {
    BinaryOp {
        dst: Variable,
        op: BinaryOp,
        left: Operand,
        right: Operand,
    },

    UnaryOp {
        dst: Variable,
        op: UnaryOp,
        src: Operand,
    },

    Param {
        src: Operand,
    },

    Call {
        dst: Variable,
        func: Operand,
        param_cnt: usize,
    },

    NewObject {
        dst: Variable,
        size: usize,
    },

    SetObject {
        obj: Variable,
        value: Operand,
        offset: usize
    },

    Index {
        dst: Variable,
        src: Operand,
        idx: Operand,
    },

    Member {
        dst: Variable,
        src: Operand,
        member: String,
    },

    Jump {
        lable: Label,
    },

    JumpIf {
        condition: Operand,
        lable: Label,
    },
}

pub struct IrBuilder {
    next_temp: usize,
    next_label: usize,
    instructions: Vec<Instruction>,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            next_temp: 0,
            next_label: 0,
            instructions: Vec::new(),
        }
    }

    pub fn new_temp(&mut self) -> usize {
        let next = self.next_temp;
        self.next_temp += 1;
        next
    }

    pub fn new_label(&mut self) -> usize {
        let next = self.next_label;
        self.next_label += 1;
        next
    }

    pub fn emit(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    pub fn vist_expr(&mut self, expr: &type_ast::Expr) -> Operand {
        match expr {
            type_ast::Expr::Literal { value, .. } => match value {
                token::Literal::Nil => Operand::ConstNil,
                token::Literal::Bool(b) => Operand::ConstBool(*b),
                token::Literal::Int(i) => Operand::ConstInt(*i),
                token::Literal::Float(f) => Operand::ConstFloat(*f),
                token::Literal::String(s) => Operand::ConstString(s.to_string()),
            },
            type_ast::Expr::Unary { op, expr, .. } => {
                let dst = Variable::Temp(self.new_temp());
                let inst = Instruction::UnaryOp {
                    dst,
                    op: *op,
                    src: self.vist_expr(expr),
                };
                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Binary {
                left, op, right, ..
            } => {
                let dst = Variable::Temp(self.new_temp());
                let inst = Instruction::BinaryOp {
                    dst,
                    op: *op,
                    left: self.vist_expr(left),
                    right: self.vist_expr(right),
                };
                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Call { func, args, .. } => {
                for arg in args {
                    let param = Instruction::Param {
                        src: self.vist_expr(arg),
                    };
                    self.emit(param);
                }

                let dst = Variable::Temp(self.new_temp());
                let inst = Instruction::Call {
                    dst,
                    func: self.vist_expr(func),
                    param_cnt: args.len(),
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Index { target, index, .. } => {
                let dst = Variable::Temp(self.new_temp());

                let inst = Instruction::Index {
                    dst,
                    src: self.vist_expr(target),
                    idx: self.vist_expr(index),
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Member { target, member, .. } => {
                let dst = Variable::Temp(self.new_temp());

                let inst = Instruction::Member{
                    dst,
                    src: self.vist_expr(target),
                    member: member.to_string()
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Struct { fields, .. } => {
                let dst = Variable::Temp(self.new_temp());
                let inst = Instruction::NewObject { dst, size: fields.len() };
                self.emit(inst);

                for field in fields {
                    let inst = Instruction::SetObject { obj: dst, value: self.vist_expr(&field.expr), offset: field.offset };
                    self.emit(inst);
                }
                Operand::Variable(dst)
            },
            type_ast::Expr::Path { segments, .. } => todo!(),
            type_ast::Expr::Method {
                this_ty,
                method_name,
                method_ty,
            } => todo!(),
        }
    }
}
