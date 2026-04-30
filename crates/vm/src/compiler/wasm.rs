use std::collections::HashMap;

use indexmap::IndexSet;
use walrus::ValType;
use walrus::ir::Value;

use crate::compiler::ConstValue;
use crate::ir::{Instruction, IrBuilder, Operand, Terminator, Variable};
use crate::semantic::functions::FunctionTable;
use crate::semantic::scope::SymbolTable;
use crate::semantic::structs::StructTable;
use crate::semantic::ty::{TypeId, TypeKind, TypeTable};
use crate::semantic::{Semantic, type_ast};

pub struct Compiler {
    pub consts: IndexSet<ConstValue>,
    pub module: walrus::Module,
    pub semantic: Semantic,
}

impl Compiler {
    fn new(semantic: Semantic) -> Self {
        Self {
            consts: IndexSet::new(),
            module: walrus::Module::default(),
            semantic,
        }
    }

    fn wasm_type(&self, type_id: TypeId) -> walrus::ValType {
        let type_kind = self.semantic.type_table.get(type_id).unwrap();
        match type_kind {
            TypeKind::Nil => todo!(),
            TypeKind::Bool => ValType::I32,
            TypeKind::Int => ValType::I64,
            TypeKind::Float => ValType::F64,
            TypeKind::String => todo!(),
            TypeKind::Struct(struct_id) => todo!(),
            TypeKind::Vec { element } => todo!(),
            TypeKind::NativeFunction { args, return_ty } => todo!(),
            TypeKind::Fn { args, return_ty } => todo!(),
        }
    }

    fn compile_fn(&mut self, f: &type_ast::FunctionDef) -> anyhow::Result<walrus::FunctionId> {
        let ir_builder = IrBuilder::new();
        let ir_f = ir_builder.visit_fn(f);
        println!("{:#?}", ir_f);
        let mut param_types = Vec::new();
        for arg in ir_f.args {
            let param = self.wasm_type(arg);
            param_types.push(param);
        }

        let results: &[walrus::ValType] = match ir_f.ret {
            Some(type_id) => &[self.wasm_type(type_id)],
            None => &[],
        };

        let mut builder =
            walrus::FunctionBuilder::new(&mut self.module.types, &param_types, &results);

        let mut block_id_map = HashMap::new();
        let mut block_ids = Vec::new();

        for block in &ir_f.blocks {
            let seq_id = builder.dangling_instr_seq(None).id();
            block_id_map.insert(block.id, seq_id);
            block_ids.push(seq_id);
            builder.func_body().instr(walrus::ir::Block { seq: seq_id });
        }
        let mut locals = Vec::new();
        for type_id in &ir_f.locals {
            let id = self.module.locals.add(self.wasm_type(*type_id));
            locals.push(id);
        }

        let mut temps = Vec::new();
        for type_id in &ir_f.temps {
            let id = self.module.locals.add(self.wasm_type(*type_id));
            temps.push(id);
        }

        let locals = Locals { locals, temps };

        for (idx, block) in ir_f.blocks.iter().enumerate() {
            let seq_id = block_ids[idx];
            
            let b = &mut builder.instr_seq(seq_id);

            for ir in &block.insts {
                match ir {
                    Instruction::BinaryOp {
                        dst,
                        op,
                        left,
                        right,
                    } => {
                        locals.get(b, left);
                        locals.get(b, right);
                        let op = match op {
                            ast::BinaryOp::Add => walrus::ir::BinaryOp::I64Add,
                            ast::BinaryOp::Subtract => walrus::ir::BinaryOp::I64Sub,
                            ast::BinaryOp::Multiply => walrus::ir::BinaryOp::I64Mul,
                            ast::BinaryOp::Divide => walrus::ir::BinaryOp::I64DivS,
                            ast::BinaryOp::Modulo => walrus::ir::BinaryOp::I64RemS,
                            ast::BinaryOp::BitAnd => walrus::ir::BinaryOp::I64And,
                            ast::BinaryOp::BitOr => walrus::ir::BinaryOp::I64Or,
                            ast::BinaryOp::BitXor => walrus::ir::BinaryOp::I64Xor,
                            ast::BinaryOp::ShiftLeft => walrus::ir::BinaryOp::I64Shl,
                            ast::BinaryOp::ShiftRight => walrus::ir::BinaryOp::I64ShrS,
                            ast::BinaryOp::Equal => walrus::ir::BinaryOp::I64Eq,
                            ast::BinaryOp::NotEqual => walrus::ir::BinaryOp::I64Ne,
                            ast::BinaryOp::Less => walrus::ir::BinaryOp::I64LtS,
                            ast::BinaryOp::LessEqual => walrus::ir::BinaryOp::I64LeS,
                            ast::BinaryOp::Greater => walrus::ir::BinaryOp::I64GtS,
                            ast::BinaryOp::GreaterEqual => walrus::ir::BinaryOp::I64GeS,
                            ast::BinaryOp::And => walrus::ir::BinaryOp::I32And,
                            ast::BinaryOp::Or => walrus::ir::BinaryOp::I32Or,
                        };
                        b.binop(op);
                        locals.set(b, dst);
                    }
                    Instruction::UnaryOp { dst, op, src } => {
                        locals.get(b, src);
                        let op = match op {
                            ast::UnaryOp::Plus => todo!(),
                            ast::UnaryOp::Minus => walrus::ir::UnaryOp::I64x2Neg,
                            ast::UnaryOp::Not => todo!(),
                            ast::UnaryOp::BitNot => todo!(),
                        };
                        b.unop(op);
                        locals.set(b, dst);
                    }
                    Instruction::GetParam { dst } => {}
                    Instruction::Param { src } => {
                        //locals.get(b, src);
                    }
                    Instruction::Call {
                        dst,
                        func,
                        param_cnt,
                    } => todo!(),
                    Instruction::NewObject { dst, size } => todo!(),
                    Instruction::SetIndex { obj, idx, src } => todo!(),
                    Instruction::SetMember { dst, src, offset } => todo!(),
                    Instruction::Index { dst, src, idx } => todo!(),
                    Instruction::Member {
                        dst,
                        src,
                        member,
                        offset,
                    } => todo!(),
                    Instruction::Load { from, to } => {
                        locals.get(b, &Operand::Variable(*from));
                        locals.set(b, to);
                    }
                    Instruction::StoreLocal { src, dst } => {
                        locals.get(b, src);
                        locals.set(b, dst);
                    }
                }
            }

            match &block.term {
                Terminator::Br {
                    cond,
                    then_block,
                    else_block,
                } => {
                    locals.get(b, cond);
                    b.br_if(block_id_map[then_block]);
                    b.br(block_id_map[else_block]);
                }
                Terminator::Jump { block } => {
                    b.br(block_id_map[block]);
                }
                Terminator::Ret(operand) => match operand {
                    Some(operand) => {
                        locals.get(b, operand);
                        b.return_();
                    }
                    None => {
                        b.return_();
                    }
                },
            }
        }

        builder.func_body().unreachable();

        let id = builder.finish(locals.locals, &mut self.module.funcs);
        println!("{:#?}", self.module.funcs.get(id));
        Ok(id)
    }
}

pub struct Locals {
    locals: Vec<walrus::LocalId>,
    temps: Vec<walrus::LocalId>,
}

impl Locals {
    pub fn get(&self, b: &mut walrus::InstrSeqBuilder, operand: &Operand) {
        match operand {
            Operand::Variable(variable) => match variable {
                Variable::Local(l) => b.local_get(self.locals[*l]),
                Variable::Global(_) => todo!(),
                Variable::Temp(t) => b.local_get(self.temps[*t]),
            },
            Operand::ConstNil => todo!(),
            Operand::ConstBool(val) => b.const_(Value::I32(*val as i32)),
            Operand::ConstInt(val) => b.const_(Value::I64(*val)),
            Operand::ConstFloat(val) => b.const_(Value::F64(*val)),
            Operand::ConstString(_) => todo!(),
        };
    }

    pub fn set(&self, b: &mut walrus::InstrSeqBuilder, variable: &Variable) {
        match variable {
            Variable::Local(l) => b.local_set(self.locals[*l]),
            Variable::Global(_) => todo!(),
            Variable::Temp(t) => b.local_set(self.temps[*t]),
        };
    }
}

pub fn compile(program: &ast::Program) -> anyhow::Result<()> {
    let mut semantic = Semantic::new();
    semantic.init_global_symbol()?;
    let type_program = semantic.analysis_type(&program)?;
    let mut compiler = Compiler::new(semantic);

    for def in &type_program.defs {
        match def {
            type_ast::Definition::StructDef(_struct_def) => {}
            type_ast::Definition::FunctionDef(function_def) => {
                if function_def.name == "main" {
                    continue;
                }
                let id = compiler.compile_fn(&function_def)?;
                compiler.module.exports.add(&function_def.name, id);
            }
            type_ast::Definition::ImplDef(impl_def) => {}
        }
    }

    compiler.module.emit_wasm_file("output.wasm")?;

    Ok(())
}
