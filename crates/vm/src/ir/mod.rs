use std::collections::HashSet;

use ast::{BinaryOp, UnaryOp};

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
pub struct Label(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Variable(Variable),
    ConstNil,
    ConstBool(bool),
    ConstInt(i64),
    ConstFloat(f64),
    ConstString(String),
    Label(Label),
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

    Jump {
        lable: Label,
    },

    JumpIfFalse {
        condition: Operand,
        lable: Label,
    },

    StoreLocal {
        src: Operand,
        dst: Variable,
    },

    Return {
        var: Option<Operand>,
    },

    Label {
        label: Label,
    },
}

pub struct IrBuilder {
    next_temp: usize,
    next_label: usize,
    loop_stack: Vec<(Label, Label)>,
    instructions: Vec<Instruction>,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            next_temp: 0,
            next_label: 0,
            loop_stack: Vec::new(),
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

    pub fn temp_count(&self) -> usize {
        self.next_temp
    }

    pub fn take(self) -> Vec<Instruction> {
        self.instructions
    }

    pub fn emit(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    pub fn visit_fn(&mut self, fn_def: &type_ast::FunctionDef) {
        for arg in fn_def.args.iter().rev() {
            self.emit(Instruction::GetParam {
                dst: Variable::from(arg.location),
            });
        }

        self.visit_block(&fn_def.body);
        println!("{:#?}", self.instructions);
    }

    pub fn visit_block(&mut self, block: &type_ast::Block) {
        for stmt in &block.statements {
            self.visit_stmt(stmt);
        }
    }

    pub fn visit_stmt(&mut self, stmt: &type_ast::BlockStmt) {
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

                let inst = Instruction::Return { var };
                self.emit(inst);
                return;
            }
            type_ast::BlockStmt::Expr(expr) => {
                self.vist_expr(expr);
            }
            type_ast::BlockStmt::Block(block) => {
                for stmt in &block.statements {
                    self.visit_stmt(stmt);
                }
            }
            type_ast::BlockStmt::If(if_stmt) => {
                let lable_else = Label(self.new_label());
                let lable_end = if if_stmt.else_branch.is_some() {
                    Label(self.new_label())
                } else {
                    lable_else
                };

                let condition = self.vist_expr(&if_stmt.condition);
                self.emit(Instruction::JumpIfFalse {
                    condition,
                    lable: lable_else,
                });

                self.visit_block(&if_stmt.then_branch);

                self.emit(Instruction::Jump { lable: lable_end });

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.emit(Instruction::Label { label: lable_else });
                    self.visit_block(else_branch);
                }

                self.emit(Instruction::Label { label: lable_end });
            }
            type_ast::BlockStmt::While(while_stmt) => {
                let lable_start = Label(self.new_label());
                let lable_end = Label(self.new_label());

                self.loop_stack.push((lable_start, lable_end));

                self.emit(Instruction::Label { label: lable_start });

                let condition = self.vist_expr(&while_stmt.condition);
                self.emit(Instruction::JumpIfFalse {
                    condition,
                    lable: lable_end,
                });

                self.visit_block(&while_stmt.block);

                self.emit(Instruction::Jump { lable: lable_start });

                self.emit(Instruction::Label { label: lable_end });
                self.loop_stack.pop();
            }
            type_ast::BlockStmt::Break => {
                let (_, end) = self.loop_stack.pop().unwrap();
                self.emit(Instruction::Jump { lable: end });
            }
            type_ast::BlockStmt::Continue => {
                let (start, _) = self.loop_stack.pop().unwrap();
                self.emit(Instruction::Jump { lable: start });
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
            type_ast::Expr::Member {
                target,
                member,
                offset,
                ..
            } => {
                let dst = Variable::Temp(self.new_temp());

                let inst = Instruction::Member {
                    dst,
                    src: self.vist_expr(target),
                    member: member.to_string(),
                    offset: *offset,
                };

                self.emit(inst);
                Operand::Variable(dst)
            }
            type_ast::Expr::Struct { fields, .. } => {
                let dst = Variable::Temp(self.new_temp());
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
            type_ast::Expr::Path { location, .. } => {
                Operand::Variable(Variable::from(*location))
            }
            type_ast::Expr::Method { location, .. } => {
                let dst = Variable::Temp(self.new_temp());

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

pub fn optimize(mut codes: Vec<Instruction>) -> Vec<Instruction> {
    let passes = [pass2, pass3, dce_pass, inline_arithmetic_pass];
    loop {
        let mut changed = false;
        for pass in passes {
            let (new_codes, new_changed) = pass(&codes);
            if new_changed {
                changed = true;
            }
            println!("优化: {}", codes.len() - new_codes.len());
            codes = new_codes;
        }
        if !changed {
            break;
        } // 直到不再有变动
    }
    println!("优化后 {:#?}", codes);
    codes
}

fn pass2(input: &[Instruction]) -> (Vec<Instruction>, bool) {
    let mut output = Vec::new();
    let mut i = 0;
    let mut changed = false;

    while i < input.len() {
        // 模式匹配：检查当前和下一条指令
        if i + 1 < input.len() {
            match (&input[i], &input[i + 1]) {
                // 优化 1: 消除重复 Load (local -> temp -> local)
                (
                    Instruction::Load { from: src, to: tmp },
                    Instruction::Load {
                        from: tmp2,
                        to: dst,
                    },
                ) if tmp == tmp2 => {
                    output.push(Instruction::Load {
                        from: *src,
                        to: *dst,
                    });
                    i += 2;
                    changed = true;
                    continue;
                }

                // 优化 2: 运算操作数内联 (Load temp + BinaryOp using temp)
                (
                    Instruction::Load { from: src, to: tmp },
                    Instruction::BinaryOp {
                        dst,
                        op,
                        left,
                        right,
                    },
                ) if Operand::Variable(*tmp) == *left => {
                    output.push(Instruction::BinaryOp {
                        dst: *dst,
                        op: *op,
                        left: Operand::Variable(*src), // 直接用 src 替换 tmp
                        right: right.clone(),
                    });
                    i += 2;
                    changed = true;
                    continue;
                }

                (
                    Instruction::Load {
                        from: from0,
                        to: Variable::Temp(to0),
                    },
                    Instruction::Load {
                        from: Variable::Temp(from1),
                        to: t01,
                    },
                ) if to0 == from1 => {
                    output.push(Instruction::Load {
                        from: *from0,
                        to: *t01,
                    });
                    i += 2;
                    changed = true;
                    continue;
                }

                (
                    Instruction::Load {
                        from: from0,
                        to: Variable::Temp(to0),
                    },
                    Instruction::Call {
                        dst,
                        func: Operand::Variable(Variable::Temp(func)),
                        param_cnt,
                    },
                ) if to0 == func => {
                    output.push(Instruction::Call {
                        dst: *dst,
                        func: Operand::Variable(*from0),
                        param_cnt: *param_cnt,
                    });
                    i += 2;
                    changed = true;
                    continue;
                }

                (
                    Instruction::Load {
                        from: from0,
                        to: Variable::Temp(to0),
                    },
                    Instruction::Return { var: Some(Operand::Variable(Variable::Temp(r))) },
                ) if to0 == r => {
                    output.push(Instruction::Return { var: Some(Operand::Variable(*from0)) });
                    i += 2;
                    changed = true;
                    continue;
                }

                (
                    Instruction::Load {
                        from: from0,
                        to: Variable::Temp(to0),
                    },
                    Instruction::StoreLocal {
                        src: Operand::Variable(Variable::Temp(src)),
                        dst,
                    },
                ) if to0 == src => {
                    output.push(Instruction::Load {
                        from: *from0,
                        to: *dst,
                    });
                    i += 2;
                    changed = true;
                    continue;
                }

                // ... 其他模式 ...
                _ => {}
            }
        }

        output.push(input[i].clone());
        i += 1;
    }
    (output, changed)
}

fn pass3(input: &[Instruction]) -> (Vec<Instruction>, bool) {
    let mut output = Vec::new();
    let mut i = 0;
    let mut changed = false;

    while i < input.len() {
        // 模式匹配：检查当前和下一条指令
        if i + 2 < input.len() {
            match (&input[i], &input[i + 1], &input[i + 2]) {
                (
                    Instruction::Load {
                        from: from0,
                        to: Variable::Temp(to0),
                    },
                    Instruction::Load {
                        from: from1,
                        to: Variable::Temp(to1),
                    },
                    Instruction::BinaryOp {
                        dst,
                        op,
                        left: Operand::Variable(Variable::Temp(left)),
                        right: Operand::Variable(Variable::Temp(right)),
                    },
                ) if (left == to0) && (right == to1) => {
                    output.push(Instruction::BinaryOp {
                        dst: *dst,
                        op: *op,
                        left: Operand::Variable(*from0),
                        right: Operand::Variable(*from1),
                    });
                    i += 3;
                    changed = true;
                    continue;
                }

                // ... 其他模式 ...
                _ => {}
            }
        }

        output.push(input[i].clone());
        i += 1;
    }
    (output, changed)
}

fn dce_pass(codes: &[Instruction]) -> (Vec<Instruction>, bool) {
    let live_vars = get_live_variables(&codes);
    let mut changed = false;

    let optimized_codes = codes
        .into_iter()
        .filter(|inst| {
            match inst {
                // 只有当目标变量不在 live_vars 中，且指令没有副作用时，才返回 false (删除)
                Instruction::Load { to, .. }
                | Instruction::BinaryOp { dst: to, .. }
                | Instruction::UnaryOp { dst: to, .. }
                | Instruction::NewObject { dst: to, .. }
                | Instruction::Index { dst: to, .. }
                | Instruction::Member { dst: to, .. }
                | Instruction::StoreLocal { dst: to, .. } => {
                    let is_dead = !live_vars.contains(to);
                    if is_dead {
                        changed = true;
                        return false; // 丢弃这条指令
                    }
                }
                // 注意：Call, SetIndex, SetMember 等具有副作用，即使结果没被用也要保留
                _ => {}
            }
            true
        })
        .cloned()
        .collect();

    (optimized_codes, changed)
}

fn get_live_variables(codes: &[Instruction]) -> HashSet<Variable> {
    let mut live_vars = HashSet::new();

    for inst in codes {
        match inst {
            // 所有读取 Operand 的地方
            Instruction::BinaryOp { left, right, .. } => {
                add_operand_to_live(&mut live_vars, left);
                add_operand_to_live(&mut live_vars, right);
            }
            Instruction::UnaryOp { src, .. }
            | Instruction::StoreLocal { src, .. }
            | Instruction::Param { src }
            | Instruction::SetIndex { src, .. }
            | Instruction::SetMember { src, .. }
            | Instruction::Index { src, .. }
            | Instruction::Member { src, .. } => {
                add_operand_to_live(&mut live_vars, src);
            }
            Instruction::Return { var: Some(src) } => {
                add_operand_to_live(&mut live_vars, src);
            }
            Instruction::Load { from, .. } => {
                live_vars.insert(*from);
            }
            Instruction::JumpIfFalse { condition, .. } => {
                add_operand_to_live(&mut live_vars, condition);
            }
            Instruction::Call { func, .. } => {
                add_operand_to_live(&mut live_vars, func);
            }
            _ => {} // 其他指令如 Jump, Label 不产生读取
        }
    }
    live_vars
}

fn add_operand_to_live(set: &mut HashSet<Variable>, op: &Operand) {
    if let Operand::Variable(v) = op {
        set.insert(*v);
    }
}

fn inline_arithmetic_pass(insts: &[Instruction]) -> (Vec<Instruction>, bool) {
    let mut output = Vec::new();
    let mut i = 0;
    let mut changed = false;

    while i < insts.len() {
        let current = &insts[i];

        // --- 尝试匹配 模式 B: BinaryOp + StoreLocal ---
        if i + 1 < insts.len() {
            if let Instruction::BinaryOp {
                dst: t_dst,
                op,
                left,
                right,
            } = current
            {
                if let Instruction::StoreLocal {
                    src: Operand::Variable(t_src),
                    dst: final_dst,
                } = &insts[i + 1]
                {
                    if t_dst == t_src {
                        // 发现模式！合并为一条指令
                        output.push(Instruction::BinaryOp {
                            dst: *final_dst,
                            op: *op,
                            left: left.clone(),
                            right: right.clone(),
                        });
                        i += 2; // 跳过这两条，处理下一组
                        changed = true;
                        continue;
                    }
                }
            }
        }

        // --- 尝试匹配 模式 A: Load + BinaryOp (处理左操作数) ---
        if i + 1 < insts.len() {
            if let Instruction::Load {
                from,
                to: t_load_to,
            } = current
            {
                if let Instruction::BinaryOp {
                    dst,
                    op,
                    left: Operand::Variable(t_op_left),
                    right,
                } = &insts[i + 1]
                {
                    if t_load_to == t_op_left {
                        output.push(Instruction::BinaryOp {
                            dst: *dst,
                            op: *op,
                            left: Operand::Variable(*from), // 直接内联源变量
                            right: right.clone(),
                        });
                        i += 2;
                        changed = true;
                        continue;
                    }
                }
            }
        }

        // 没有匹配到优化模式，原样保留
        output.push(current.clone());
        i += 1;
    }

    (output, changed)
}
