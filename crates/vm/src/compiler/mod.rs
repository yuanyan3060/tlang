use std::collections::HashMap;

use indexmap::IndexSet;
use ordered_float::OrderedFloat;

use crate::bytecode::{ByteCode, Loc};
use crate::ir::{Instruction, IrBuilder, Operand, Terminator};
use crate::package::{Function, Package};
use crate::semantic::structs::StructTable;
use crate::semantic::ty::{TypeId, TypeTable};
use crate::semantic::{self, Semantic, type_ast};
use crate::value::Value;

pub struct Compiler {
    pub consts: IndexSet<ConstValue>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum ConstValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(OrderedFloat<f64>),
    String(String),
}

impl From<ConstValue> for Value {
    fn from(value: ConstValue) -> Self {
        match value {
            ConstValue::Nil => Value::Nil,
            ConstValue::Bool(b) => Value::Bool(b),
            ConstValue::Int(i) => Value::Int(i),
            ConstValue::Float(f) => Value::Float(f.into()),
            ConstValue::String(_) => todo!(),
        }
    }
}

impl Compiler {
    fn new() -> Self {
        Self {
            consts: IndexSet::new(),
        }
    }

    fn compile_fn(&mut self, f: &type_ast::FunctionDef) -> anyhow::Result<Function> {
        let ir_builder = IrBuilder::new();
        let f = ir_builder.visit_fn(f);

        println!("blocks {:#?}", f.blocks);
        let mut codes = Vec::new();
        let mut block_starts = HashMap::new();
        for block in f.blocks {
            block_starts.insert(block.id.0 as u32, codes.len() as u32);
            self.compile_irs(&block.insts, &mut codes)?;

            match block.term {
                Terminator::Br {
                    cond,
                    then_block,
                    else_block,
                } => match cond {
                    Operand::Variable(variable) => {
                        codes.push(ByteCode::Br {
                            cond: Loc::from(variable),
                            then_offset: then_block.0 as u32,
                            else_offset: else_block.0 as u32,
                        });
                    }
                    Operand::ConstBool(b) => {
                        if b {
                            codes.push(ByteCode::Jump {
                                offset: then_block.0 as u32,
                            });
                        } else {
                            codes.push(ByteCode::Jump {
                                offset: else_block.0 as u32,
                            });
                        }
                    }
                    _ => unreachable!(),
                },
                Terminator::Jump { block } => codes.push(ByteCode::Jump {
                    offset: block.0 as u32,
                }),
                Terminator::Ret(operand) => codes.push(ByteCode::Return {
                    src: operand.as_ref().map(|x| self.operand_to_loc(&x)),
                }),
            }
        }

        for code in &mut codes {
            match code {
                ByteCode::Br {
                    then_offset,
                    else_offset,
                    ..
                } => {
                    *then_offset = block_starts[then_offset];
                    *else_offset = block_starts[else_offset];
                }
                ByteCode::Jump { offset } => *offset = block_starts[offset],
                _ => continue,
            }
        }

        println!("compile {} {:#?}", f.name, codes);

        Ok(Function::Custom {
            name: f.name.to_string(),
            codes,
            local_var_cnt: f.local_cnt as u32,
            temp_var_cnt: f.temp_cnt as u32,
        })
    }

    fn intern_const(&mut self, value: ConstValue) -> Loc {
        match self.consts.get_index_of(&value) {
            Some(idx) => Loc::from_const(idx),
            None => {
                let idx = self.consts.len();
                self.consts.insert(value);
                Loc::from_const(idx)
            }
        }
    }

    fn operand_to_loc(&mut self, operand: &Operand) -> Loc {
        match operand {
            Operand::Variable(variable) => Loc::from(*variable),
            Operand::ConstNil => self.intern_const(ConstValue::Nil),
            Operand::ConstBool(b) => self.intern_const(ConstValue::Bool(*b)),
            Operand::ConstInt(i) => self.intern_const(ConstValue::Int(*i)),
            Operand::ConstFloat(f) => self.intern_const(ConstValue::Float((*f).into())),
            Operand::ConstString(s) => self.intern_const(ConstValue::String(s.to_string())),
        }
    }

    fn compile_irs(
        &mut self,
        irs: &[Instruction],
        codes: &mut Vec<ByteCode>,
    ) -> anyhow::Result<()> {
        for ir in irs {
            match ir {
                Instruction::BinaryOp {
                    dst,
                    op,
                    left,
                    right,
                } => {
                    let code = match op {
                        ast::BinaryOp::Add => ByteCode::Add {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Subtract => ByteCode::Sub {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Multiply => ByteCode::Mul {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Divide => ByteCode::Div {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Modulo => ByteCode::Mod {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::BitAnd => ByteCode::BitAnd {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::BitOr => ByteCode::BitOr {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::BitXor => ByteCode::BitXor {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::ShiftLeft => ByteCode::Shl {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::ShiftRight => ByteCode::Shr {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Equal => ByteCode::Eq {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::NotEqual => ByteCode::Ne {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Less => ByteCode::Lt {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::LessEqual => ByteCode::Le {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Greater => ByteCode::Gt {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::GreaterEqual => ByteCode::Ge {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::And => ByteCode::And {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                        ast::BinaryOp::Or => ByteCode::Or {
                            dst: Loc::from(*dst),
                            left: self.operand_to_loc(left),
                            right: self.operand_to_loc(right),
                        },
                    };
                    codes.push(code);
                }
                Instruction::UnaryOp { dst, op, src } => {
                    let code = match op {
                        ast::UnaryOp::Plus => ByteCode::Pos {
                            dst: Loc::from(*dst),
                            src: self.operand_to_loc(src),
                        },
                        ast::UnaryOp::Minus => ByteCode::Neg {
                            dst: Loc::from(*dst),
                            src: self.operand_to_loc(src),
                        },
                        ast::UnaryOp::Not => ByteCode::Not {
                            dst: Loc::from(*dst),
                            src: self.operand_to_loc(src),
                        },
                        ast::UnaryOp::BitNot => ByteCode::BitNot {
                            dst: Loc::from(*dst),
                            src: self.operand_to_loc(src),
                        },
                    };
                    codes.push(code);
                }
                Instruction::GetParam { dst } => {
                    codes.push(ByteCode::GetParam {
                        dst: Loc::from(*dst),
                    });
                }
                Instruction::Param { src } => {
                    codes.push(ByteCode::Param {
                        src: self.operand_to_loc(src),
                    });
                }
                Instruction::Call {
                    dst,
                    func,
                    param_cnt,
                } => {
                    codes.push(ByteCode::Call {
                        dst: Loc::from(*dst),
                        func: self.operand_to_loc(func),
                        param_cnt: *param_cnt as u16,
                    });
                }
                Instruction::NewObject { dst, size } => {
                    codes.push(ByteCode::NewObject {
                        dst: Loc::from(*dst),
                        size: *size as u32,
                    });
                }
                Instruction::SetIndex { obj: dst, idx, src } => {
                    codes.push(ByteCode::SetIndex {
                        dst: Loc::from(*dst),
                        idx: self.operand_to_loc(idx),
                        src: self.operand_to_loc(src),
                    });
                }
                Instruction::SetMember { dst, src, offset } => {
                    codes.push(ByteCode::SetMember {
                        dst: Loc::from(*dst),
                        offset: *offset as u16,
                        src: self.operand_to_loc(src),
                    });
                }
                Instruction::Index { dst, src, idx } => {
                    codes.push(ByteCode::Index {
                        dst: Loc::from(*dst),
                        idx: self.operand_to_loc(idx),
                        src: self.operand_to_loc(src),
                    });
                }
                Instruction::Member {
                    dst, src, offset, ..
                } => {
                    codes.push(ByteCode::Member {
                        dst: Loc::from(*dst),
                        offset: *offset as u16,
                        src: self.operand_to_loc(src),
                    });
                }
                Instruction::Load { from, to } => {
                    codes.push(ByteCode::Load {
                        from: Loc::from(*from),
                        to: Loc::from(*to),
                    });
                }
                Instruction::StoreLocal { src, dst } => {
                    codes.push(ByteCode::Load {
                        from: self.operand_to_loc(src),
                        to: Loc::from(*dst),
                    });
                }
            }
        }

        Ok(())
    }
}

pub fn compile(program: &ast::Program) -> anyhow::Result<Package> {
    let mut semantic = Semantic::new();
    semantic.init_global_symbol()?;
    let type_program = semantic.analysis_type(&program)?;
    let mut compiler = Compiler::new();
    let mut functions = Vec::new();
    let mut global = vec![Value::Nil; semantic.symbol_table.global_count()];

    let mut entry_function = None;
    for def in &type_program.defs {
        match def {
            type_ast::Definition::StructDef(_struct_def) => {}
            type_ast::Definition::FunctionDef(function_def) => {
                let f = compiler.compile_fn(&function_def)?;
                let idx = functions.len();
                global[function_def.idx] = Value::Fn(idx as u32);
                functions.push(f);

                if function_def.name == "main" {
                    entry_function = Some(idx)
                }
            }
            type_ast::Definition::ImplDef(impl_def) => {
                for f in &impl_def.functions {
                    let idx = functions.len();
                    let f = match f {
                        type_ast::AssociatedFunction::Function(function_def) => {
                            let f = compiler.compile_fn(function_def)?;
                            global[function_def.idx] = Value::Fn(idx as u32);
                            f
                        }
                        type_ast::AssociatedFunction::Method(function_def) => {
                            let f = compiler.compile_fn(function_def)?;
                            global[function_def.idx] = Value::Fn(idx as u32);
                            f
                        }
                    };
                    functions.push(f);
                }
            }
        }
    }

    let constants = compiler.consts.into_iter().collect();

    Ok(Package {
        constants,
        global,
        structs: semantic.struct_table,
        functions,
        entry_function: entry_function.unwrap(),
    })
}
