use crate::package::{Function, Package};
use crate::semantic::structs::StructTable;
use crate::semantic::ty::{TypeId, TypeTable};
use crate::semantic::{self, Semantic, type_ast};

pub struct Compiler {
    pub type_table: TypeTable,
    pub struct_table: StructTable,
}

impl Compiler {
    fn compile_fn(
        &mut self,
        this: Option<TypeId>,
        f: &type_ast::FunctionDef,
    ) -> anyhow::Result<Function> {
        todo!()
    }

    fn compile_block(
        &mut self,
        this: Option<TypeId>,
        f: &type_ast::FunctionDef,
    ) -> anyhow::Result<Function> {
        todo!()
    }
}

pub fn compile(program: &ast::Program) -> anyhow::Result<Package> {
    let mut semantic = Semantic::new();
    let type_program = semantic.analysis_type(&program)?;
    let mut compiler = Compiler {
        type_table: semantic.type_table,
        struct_table: semantic.struct_table,
    };

    for def in type_program.defs {
        match def {
            type_ast::Definition::StructDef(struct_def) => {}
            type_ast::Definition::FunctionDef(f) => {
                compiler.compile_fn(None, &f)?;
            }
            type_ast::Definition::ImplDef(impl_def) => {
                for f in impl_def.functions {
                    match f {
                        type_ast::AssociatedFunction::Function(f) => {
                            compiler.compile_fn(None, &f)?;
                        },
                        type_ast::AssociatedFunction::Method(f) => {
                            compiler.compile_fn(Some(impl_def.ty), &f)?;
                        },
                    }
                }
            }
        }
    }

    todo!()
}
