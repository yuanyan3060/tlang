use ast::{BinaryOp, UnaryOp};

use crate::semantic::ty::TypeId;
use crate::semantic::{scope::Location, type_ast};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variable {
    Local(usize),
    Global(usize),
    Temp(usize),
}

impl From<Location> for Variable {
    fn from(value: Location) -> Self {
        match value {
            Location::Local(idx) => Self::Local(idx),
            Location::Global(idx) => Self::Global(idx),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Variable(Variable),
    ConstNil,
    ConstBool(bool),
    ConstInt(i64),
    ConstFloat(f64),
    ConstString(String),
}

impl Operand {
    pub fn as_var(&self) -> Option<Variable> {
        match self {
            Self::Variable(var) => Some(*var),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
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

    GetParam {
        dst: Variable,
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

    SetIndex {
        obj: Variable,
        idx: Operand,
        src: Operand,
    },

    SetMember {
        dst: Variable,
        src: Operand,
        offset: usize,
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
        offset: usize,
    },

    Load {
        from: Variable,
        to: Variable,
    },

    StoreLocal {
        src: Operand,
        dst: Variable,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<TypeId>,
    pub blocks: Vec<BasicBlock>,
    pub temps: Vec<TypeId>,
    pub locals: Vec<TypeId>,
    pub ret: Option<TypeId>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub insts: Vec<Instruction>,
    pub term: Terminator,
}

#[derive(Debug, Clone)]
pub struct IrBasicBlock {
    pub id: BlockId,
    pub insts: Vec<Instruction>,
    pub term: Option<Terminator>,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Br {
        cond: Operand,
        then_block: BlockId,
        else_block: BlockId,
    },
    Jump {
        block: BlockId,
    },
    Ret(Option<Operand>),
}

pub struct IrBuilder {
    temps: Vec<TypeId>,
    loop_stack: Vec<(BlockId, BlockId)>,
    blocks: Vec<IrBasicBlock>,
    curr: BlockId,
}

impl IrBuilder {
    pub fn new() -> Self {
        let curr = BlockId(0);
        let block = IrBasicBlock {
            id: curr,
            insts: Vec::new(),
            term: None,
        };

        Self {
            temps: Vec::new(),
            loop_stack: Vec::new(),
            blocks: vec![block],
            curr,
        }
    }

    pub fn add_temp(&mut self, type_id: TypeId) -> usize {
        let id = self.temps.len();
        self.temps.push(type_id);
        id
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(IrBasicBlock {
            id,
            insts: Vec::new(),
            term: None,
        });
        id
    }

    pub fn set_curr(&mut self, id: BlockId) {
        self.curr = id
    }

    pub fn emit(&mut self, inst: Instruction) {
        self.curr_block().insts.push(inst);
    }

    pub fn curr_block(&mut self) -> &mut IrBasicBlock {
        &mut self.blocks[self.curr.0]
    }

    pub fn emit_term(&mut self, term: Terminator) {
        self.curr_block().term = Some(term)
    }

    pub fn has_term(&mut self) -> bool {
        self.curr_block().term.is_some()
    }

    pub fn visit_fn(mut self, fn_def: &type_ast::FunctionDef) -> Function {
        for arg in fn_def.args.iter().rev() {
            self.emit(Instruction::GetParam {
                dst: Variable::from(arg.location),
            });
        }

        self.visit_block(&fn_def.body);

        if !self.has_term() && fn_def.return_type.is_none() {
            self.emit_term(Terminator::Ret(None));
        }

        let mut blocks = Vec::new();

        for block in self.blocks {
            let block = BasicBlock {
                id: block.id,
                insts: block.insts,
                term: block.term.unwrap(),
            };

            blocks.push(block);
        }

        let mut args = Vec::new();

        for arg in &fn_def.args {
            args.push(arg.type_);
        }
        Function {
            name: fn_def.name.to_string(),
            args,
            blocks,
            temps: self.temps,
            locals: fn_def.locals.clone(),
            ret: fn_def.return_type,
        }
    }

    pub fn visit_block(&mut self, block: &type_ast::Block) {
        for stmt in &block.statements {
            match stmt {
                type_ast::BlockStmt::Let(let_stmt) => {
                    let inst = Instruction::StoreLocal {
                        src: self.vist_expr(&let_stmt.expr),
                        dst: Variable::from(let_stmt.location),
                    };
                    self.emit(inst);
                }
                type_ast::BlockStmt::Assign(assign_stmt) => match &assign_stmt.target {
                    type_ast::Expr::Index { target, index, .. } => {
                        let inst = Instruction::SetIndex {
                            obj: self.vist_expr(&target).as_var().unwrap(),
                            idx: self.vist_expr(&index),
                            src: self.vist_expr(&assign_stmt.expr),
                        };
                        self.emit(inst);
                    }
                    type_ast::Expr::Member { target, offset, .. } => {
                        let inst = Instruction::SetMember {
                            dst: self.vist_expr(target).as_var().unwrap(),
                            src: self.vist_expr(&assign_stmt.expr),
                            offset: *offset,
                        };
                        self.emit(inst);
                    }
                    type_ast::Expr::Path { location, .. } => {
                        let inst = Instruction::StoreLocal {
                            src: self.vist_expr(&assign_stmt.expr),
                            dst: Variable::from(*location),
                        };
                        self.emit(inst);
                    }
                    _ => todo!(),
                },
                type_ast::BlockStmt::Return(return_stmt) => {
                    let var = match &return_stmt.expr {
                        Some(expr) => Some(self.vist_expr(expr)),
                        None => None,
                    };

                    self.emit_term(Terminator::Ret(var));
                    return;
                }
                type_ast::BlockStmt::Expr(expr) => {
                    self.vist_expr(expr);
                }
                type_ast::BlockStmt::Block(block) => {
                    self.visit_block(block);
                }
                type_ast::BlockStmt::If(if_stmt) => {
                    let then_block = self.new_block();
                    let else_block = self.new_block();
                    let merge_block = self.new_block();

                    let cond = self.vist_expr(&if_stmt.condition);
                    self.emit_term(Terminator::Br {
                        cond,
                        then_block,
                        else_block,
                    });

                    self.set_curr(then_block);
                    self.visit_block(&if_stmt.then_branch);
                    if !self.has_term() {
                        self.emit_term(Terminator::Jump { block: merge_block });
                    }

                    self.set_curr(else_block);
                    if let Some(else_branch) = &if_stmt.else_branch {
                        self.visit_block(else_branch);
                    }
                    if !self.has_term() {
                        self.emit_term(Terminator::Jump { block: merge_block });
                    }

                    self.set_curr(merge_block);
                }
                type_ast::BlockStmt::While(while_stmt) => {
                    let cond = self.new_block();
                    let body = self.new_block();
                    let exit = self.new_block();

                    self.emit_term(Terminator::Jump { block: cond });

                    self.set_curr(cond);
                    let condition = self.vist_expr(&while_stmt.condition);
                    self.emit_term(Terminator::Br {
                        cond: condition,
                        then_block: body,
                        else_block: exit,
                    });

                    self.set_curr(body);
                    self.loop_stack.push((cond, exit));
                    self.visit_block(&while_stmt.block);
                    self.loop_stack.pop();

                    if !self.has_term() {
                        self.emit_term(Terminator::Jump { block: cond });
                    }

                    self.set_curr(exit);
                }
                type_ast::BlockStmt::Break => {
                    let (_, exit) = self.loop_stack.last().unwrap();
                    self.emit_term(Terminator::Jump { block: *exit });
                }
                type_ast::BlockStmt::Continue => {
                    let (cond, _) = self.loop_stack.last().unwrap();
                    self.emit_term(Terminator::Jump { block: *cond });
                }
            }
        }
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
            type_ast::Expr::Unary { op, expr, ty, .. } => {
                let dst = Variable::Temp(self.add_temp(*ty));
                let inst = Instruction::UnaryOp {
                    dst,
                    op: *op,
                    src: self.vist_expr(expr),
                };
                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Binary {
                left, op, right, ty, ..
            } => {
                let dst = Variable::Temp(self.add_temp(*ty));
                let inst = Instruction::BinaryOp {
                    dst,
                    op: *op,
                    left: self.vist_expr(left),
                    right: self.vist_expr(right),
                };
                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Call { func, args, ty, .. } => {
                for arg in args {
                    let param = Instruction::Param {
                        src: self.vist_expr(arg),
                    };
                    self.emit(param);
                }

                let dst = Variable::Temp(self.add_temp(*ty));
                let inst = Instruction::Call {
                    dst,
                    func: self.vist_expr(func),
                    param_cnt: args.len(),
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Index { target, index, ty, .. } => {
                let dst = Variable::Temp(self.add_temp(*ty));

                let inst = Instruction::Index {
                    dst,
                    src: self.vist_expr(target),
                    idx: self.vist_expr(index),
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Member {
                target,
                member,
                offset,
                member_ty,
                ..
            } => {
                let dst = Variable::Temp(self.add_temp(*member_ty));

                let inst = Instruction::Member {
                    dst,
                    src: self.vist_expr(target),
                    member: member.to_string(),
                    offset: *offset,
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Struct { fields, struct_ty, .. } => {
                let dst = Variable::Temp(self.add_temp(*struct_ty));
                let inst = Instruction::NewObject {
                    dst,
                    size: fields.len(),
                };
                self.emit(inst);

                for field in fields {
                    let inst = Instruction::SetMember {
                        dst,
                        src: self.vist_expr(&field.expr),
                        offset: field.offset,
                    };
                    self.emit(inst);
                }
                Operand::Variable(dst)
            }
            type_ast::Expr::Path { location, .. } => Operand::Variable(Variable::from(*location)),
            type_ast::Expr::Method { location, method_ty, .. } => {
                let dst = Variable::Temp(self.add_temp(*method_ty));

                let inst = Instruction::Load {
                    to: dst,
                    from: Variable::from(*location),
                };
                self.emit(inst);

                Operand::Variable(dst)
            }
        }
    }
}
