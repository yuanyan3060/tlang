use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Debug, Display};

use ast::{BinaryOp, Type, UnaryOp};

use crate::semantic::functions::FunctionTable;
use crate::semantic::scope::{Location, SymbolTable};
use crate::semantic::structs::StructTable;
use crate::semantic::ty::{GenericFn, GenericSlot, TypeId, TypeKind, TypeSlot, TypeTable};

pub mod functions;
pub mod scope;
pub mod structs;
pub mod ty;
pub mod type_ast;

pub struct Semantic {
    pub type_table: TypeTable,
    pub struct_table: StructTable,
    pub function_table: FunctionTable,
    pub symbol_table: SymbolTable,
}

impl Semantic {
    pub fn new() -> Self {
        Self {
            type_table: TypeTable::new(),
            struct_table: StructTable::new(),
            function_table: FunctionTable::new(),
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn init_global_symbol(&mut self) -> Result<(), SemanticError> {
        self.symbol_table.insert_generic_fn(
            "print",
            GenericFn {
                args: vec![TypeSlot::Dyn(GenericSlot(0))],
                return_ty: None,
            },
        )?;
        self.symbol_table.insert_generic_fn(
            "Vec::new",
            GenericFn {
                args: vec![TypeSlot::Dyn(GenericSlot(0))],
                return_ty: None,
            },
        )?;
        self.symbol_table.insert_generic_fn(
            "Vec::push",
            GenericFn {
                args: vec![TypeSlot::Dyn(GenericSlot(0))],
                return_ty: None,
            },
        )?;
        self.symbol_table.insert_generic_fn(
            "Vec::len",
            GenericFn {
                args: vec![TypeSlot::Dyn(GenericSlot(0))],
                return_ty: None,
            },
        )?;
        Ok(())
    }

    pub fn method(
        &mut self,
        this: TypeId,
        name: &str,
    ) -> Result<(TypeId, Location), SemanticError> {
        match self.type_table.get(this).unwrap() {
            TypeKind::Nil => todo!(),
            TypeKind::Bool => todo!(),
            TypeKind::Int => todo!(),
            TypeKind::Float => todo!(),
            TypeKind::String => todo!(),
            TypeKind::Struct(struct_id) => todo!(),
            TypeKind::Vec { element } => match name {
                "new" => {
                    let type_id = self.type_table.intern(TypeKind::NativeFunction {
                        args: Vec::new(),
                        return_ty: this,
                    });
                    let location = self.symbol_table.lookup("Vec::new").unwrap().location;
                    Ok((type_id, location))
                }
                "push" => {
                    let type_id = self.type_table.intern(TypeKind::NativeFunction {
                        args: vec![this, *element],
                        return_ty: TypeId::NIL,
                    });
                    let location = self.symbol_table.lookup("Vec::push").unwrap().location;
                    Ok((type_id, location))
                }
                "len" => {
                    let type_id = self.type_table.intern(TypeKind::NativeFunction {
                        args: vec![this],
                        return_ty: TypeId::INT,
                    });
                    let location = self.symbol_table.lookup("Vec::len").unwrap().location;
                    Ok((type_id, location))
                }
                _ => {
                    panic!()
                }
            },
            TypeKind::NativeFunction { args, return_ty } => todo!(),
            TypeKind::Fn { args, return_ty } => todo!(),
        }
    }

    pub fn type_id(&mut self, ty: &ast::Type) -> Result<TypeId, SemanticError> {
        if ty.segments.len() != 1 {
            unimplemented!("ty.segments.len() == {}", ty.segments.len());
        }

        let segment = &ty.segments[0];

        let id = if segment.ident == "Vec" {
            assert_eq!(segment.args.len(), 1);
            let element = self.type_id(&segment.args[0])?;
            self.type_table.intern(TypeKind::Vec { element })
        } else {
            assert_eq!(segment.args.len(), 0);
            match segment.ident.as_str() {
                "nil" => self.type_table.intern(TypeKind::Nil),
                "bool" => self.type_table.intern(TypeKind::Bool),
                "int" => self.type_table.intern(TypeKind::Int),
                "float" => self.type_table.intern(TypeKind::Float),
                "str" => self.type_table.intern(TypeKind::String),
                _ => {
                    println!("{}", segment.ident);
                    let struct_id = self.struct_table.id(&segment.ident).unwrap();
                    self.type_table.intern(TypeKind::Struct(struct_id))
                }
            }
        };

        Ok(id)
    }

    pub fn scan_struct_defs(&mut self, program: &ast::Program) -> Result<(), SemanticError> {
        for def in &program.defs {
            match def {
                ast::Definition::StructDef(struct_def) => {
                    self.struct_table.insert(type_ast::StructDef {
                        name: struct_def.name.to_string(),
                        fields: Vec::new(),
                    })?;
                }
                ast::Definition::FunctionDef(_function_def) => {}
                ast::Definition::ImplDef(_impl_def) => {}
            }
        }

        for def in &program.defs {
            match def {
                ast::Definition::StructDef(struct_def) => {
                    let mut fields = Vec::new();
                    for field in &struct_def.fields {
                        let ty = self.type_id(&field.type_)?;
                        let field = type_ast::Field {
                            name: field.name.to_string(),
                            type_: ty,
                        };

                        fields.push(field);
                    }

                    self.struct_table
                        .get_by_name_mut(&struct_def.name)
                        .unwrap()
                        .fields = fields;
                }
                ast::Definition::FunctionDef(_function_def) => {}
                ast::Definition::ImplDef(_impl_def) => {}
            }
        }
        Ok(())
    }

    pub fn scan_function_defs(&mut self, program: &ast::Program) -> Result<(), SemanticError> {
        for def in &program.defs {
            match def {
                ast::Definition::StructDef(struct_def) => {}
                ast::Definition::FunctionDef(function_def) => {
                    let mut args = Vec::new();
                    for arg in &function_def.args {
                        let arg_ty = self.type_id(&arg.type_)?;
                        args.push(arg_ty);
                    }
                    let return_ty = match &function_def.return_type {
                        Some(return_ty) => self.type_id(return_ty)?,
                        None => TypeId::NIL,
                    };
                    let type_kind = TypeKind::Fn { args, return_ty };
                    let type_id = self.type_table.intern(type_kind);
                    self.symbol_table.insert(&function_def.name, type_id)?;
                }
                ast::Definition::ImplDef(_impl_def) => {}
            }
        }

        Ok(())
    }

    pub fn analysis_type(
        &mut self,
        program: &ast::Program,
    ) -> Result<type_ast::Program, SemanticError> {
        self.scan_struct_defs(program)?;
        self.scan_function_defs(program)?;

        let mut type_program = type_ast::Program { defs: Vec::new() };

        for def in &program.defs {
            let type_def = match def {
                ast::Definition::StructDef(struct_def) => {
                    let mut type_fields = Vec::new();

                    for field in &struct_def.fields {
                        let type_field = type_ast::Field {
                            name: field.name.to_string(),
                            type_: self.type_id(&field.type_)?,
                        };

                        type_fields.push(type_field);
                    }

                    type_ast::Definition::StructDef(type_ast::StructDef {
                        name: struct_def.name.to_string(),
                        fields: type_fields,
                    })
                }
                ast::Definition::FunctionDef(fn_def) => {
                    type_ast::Definition::FunctionDef(analysis_func(self, fn_def, None)?)
                }
                ast::Definition::ImplDef(impl_def) => {
                    let this = self.type_id(&impl_def.ty)?;
                    let mut type_fns = Vec::new();
                    for fn_def in &impl_def.functions {
                        let type_fn = match fn_def {
                            ast::AssociatedFunction::Function(fn_def) => {
                                type_ast::AssociatedFunction::Function(analysis_func(
                                    self, fn_def, None,
                                )?)
                            }
                            ast::AssociatedFunction::Method(fn_def) => {
                                type_ast::AssociatedFunction::Method(analysis_func(
                                    self,
                                    fn_def,
                                    Some(this),
                                )?)
                            }
                        };

                        type_fns.push(type_fn);
                    }

                    type_ast::Definition::ImplDef(type_ast::ImplDef {
                        ty: this,
                        functions: type_fns,
                    })
                }
            };

            type_program.defs.push(type_def);
        }
        Ok(type_program)
    }
}

fn analysis_expr(
    semantic: &mut Semantic,
    expr: &ast::Expr,
) -> Result<type_ast::Expr, SemanticError> {
    let type_expr = match expr {
        ast::Expr::Literal(literal) => {
            let ty = match literal {
                token::Literal::Nil => TypeId::NIL,
                token::Literal::Bool(_) => TypeId::BOOL,
                token::Literal::Int(_) => TypeId::INT,
                token::Literal::Float(_) => TypeId::FLOAT,
                token::Literal::String(_) => TypeId::STRING,
            };

            type_ast::Expr::Literal {
                value: literal.clone(),
                ty,
            }
        }
        ast::Expr::Unary { op, expr } => {
            let expr = analysis_expr(semantic, expr)?;
            let ty = *expr.ty();
            type_ast::Expr::Unary {
                op: *op,
                expr: Box::new(expr),
                ty,
            }
        }
        ast::Expr::Binary { left, op, right } => {
            let left = analysis_expr(semantic, left)?;
            let right = analysis_expr(semantic, right)?;

            if left.ty() != right.ty() {
                return Err(SemanticError::TypeMistmatch);
            }

            let ty = match op {
                BinaryOp::Add => *left.ty(),
                BinaryOp::Subtract => *left.ty(),
                BinaryOp::Multiply => *left.ty(),
                BinaryOp::Divide => *left.ty(),
                BinaryOp::Modulo => *left.ty(),
                BinaryOp::BitAnd => *left.ty(),
                BinaryOp::BitOr => *left.ty(),
                BinaryOp::BitXor => *left.ty(),
                BinaryOp::ShiftLeft => *left.ty(),
                BinaryOp::ShiftRight => *left.ty(),
                BinaryOp::Equal => TypeId::BOOL,
                BinaryOp::NotEqual => TypeId::BOOL,
                BinaryOp::Less => TypeId::BOOL,
                BinaryOp::LessEqual => TypeId::BOOL,
                BinaryOp::Greater => TypeId::BOOL,
                BinaryOp::GreaterEqual => TypeId::BOOL,
                BinaryOp::And => {
                    if *left.ty() != TypeId::BOOL {
                        return Err(SemanticError::TypeMistmatch);
                    }
                    TypeId::BOOL
                }
                BinaryOp::Or => {
                    if *left.ty() != TypeId::BOOL {
                        return Err(SemanticError::TypeMistmatch);
                    }
                    TypeId::BOOL
                }
            };

            type_ast::Expr::Binary {
                left: Box::new(left),
                op: *op,
                right: Box::new(right),
                ty,
            }
        }
        ast::Expr::Call { func, args } => {
            let mut typed_args = Vec::new();

            for arg in args {
                let arg = analysis_expr(semantic, arg)?;
                typed_args.push(arg);
            }

            let func = match func.as_ref() {
                ast::Expr::Path { segments } => {
                    println!("{:?}", segments);

                    let segment = &segments[0];

                    let type_segments = analysis_path(semantic, segments)?;

                    let (ty, loc) = match semantic.symbol_table.lookup(&segment.ident) {
                        Some(symbol) => match &symbol.kind {
                            scope::SymbolKind::Normal { type_id } => (*type_id, symbol.location),
                            scope::SymbolKind::GenericFn { func } => {
                                assert_eq!(segments.len(), 1);
                                assert!(segment.args.is_empty());
                                let input_args =
                                    typed_args.iter().map(|x| *x.ty()).collect::<Vec<_>>();
                                let type_kind = func.monomorphization(&input_args)?;
                                (semantic.type_table.intern(type_kind), symbol.location)
                            }
                        },
                        None if segments.len() == 2 => {
                            let (method, this) = segments.split_last().unwrap();
                            let type_id = semantic.type_id(&Type {
                                segments: this.to_vec(),
                            })?;
                            assert!(method.args.is_empty());
                            semantic.method(type_id, &method.ident)?
                        }
                        None => {
                            panic!("can not find symbol {:?}", segments);
                        }
                    };

                    type_ast::Expr::Path {
                        segments: type_segments,
                        location: loc,
                        ty,
                    }
                }
                ast::Expr::Member { target, member } => {
                    let target = analysis_expr(semantic, target)?;
                    let target_ty = *target.ty();
                    let (method_ty, location) = semantic.method(target_ty, &member)?;
                    typed_args.insert(0, target);
                    type_ast::Expr::Method {
                        this_ty: target_ty,
                        method_name: member.to_string(),
                        method_ty,
                        location,
                    }
                }
                _ => analysis_expr(semantic, func)?,
            };

            let func_def = semantic.type_table.get(*func.ty()).unwrap();
            let (args_def, return_ty) = func_def.as_callable().unwrap();
            assert_eq!(args_def.len(), typed_args.len());

            for (arg, ty) in typed_args.iter().zip(args_def) {
                assert_eq!(arg.ty(), ty);
            }

            type_ast::Expr::Call {
                func: Box::new(func),
                args: typed_args,
                ty: return_ty,
            }
        }
        ast::Expr::Index { target, index } => {
            let target = analysis_expr(semantic, target)?;
            let index = analysis_expr(semantic, index)?;

            assert_eq!(*index.ty(), TypeId::INT);

            let target_kind = semantic.type_table.get(*target.ty()).unwrap();

            let ty = match target_kind {
                TypeKind::Vec { element } => *element,
                _ => todo!(),
            };

            type_ast::Expr::Index {
                target: Box::new(target),
                index: Box::new(index),
                ty,
            }
        }
        ast::Expr::Member { target, member } => {
            let target = analysis_expr(semantic, target)?;

            let target_kind = semantic.type_table.get(*target.ty()).unwrap();

            let (offset, field) = match target_kind {
                TypeKind::Struct(id) => {
                    let struct_def = semantic.struct_table.get(*id).unwrap();
                    struct_def
                        .fields
                        .iter()
                        .enumerate()
                        .find(|(_, x)| &x.name == member)
                        .unwrap()
                }
                _ => {
                    println!("{:?}", expr);
                    todo!()
                }
            };

            type_ast::Expr::Member {
                target: Box::new(target),
                member: member.to_string(),
                offset,
                member_ty: field.type_,
            }
        }
        ast::Expr::Struct { struct_ty, fields } => {
            let struct_ty = semantic.type_id(struct_ty)?;

            let mut type_fields = Vec::new();
            let mut init_fields = HashSet::new();
            let field_offsets = {
                let struct_kind = semantic.type_table.get(struct_ty).unwrap();

                let struct_def = match struct_kind {
                    TypeKind::Struct(id) => semantic.struct_table.get(*id).unwrap(),
                    _ => todo!(),
                };

                struct_def
                    .fields
                    .iter()
                    .map(|x| &x.name)
                    .enumerate()
                    .map(|(idx, name)| (name.to_string(), idx))
                    .collect::<HashMap<_, _>>()
            };

            for field in fields {
                let expr = analysis_expr(semantic, &field.expr)?;

                if !init_fields.insert(&field.name) {
                    panic!("field aleady set")
                }

                let field = type_ast::FieldAssign {
                    name: field.name.to_string(),
                    offset: field_offsets[&field.name],
                    expr,
                };

                type_fields.push(field);
            }

            let struct_kind = semantic.type_table.get(struct_ty).unwrap();

            let struct_def = match struct_kind {
                TypeKind::Struct(id) => semantic.struct_table.get(*id).unwrap(),
                _ => todo!(),
            };

            assert_eq!(init_fields.len(), struct_def.fields.len());
            for field in &struct_def.fields {
                assert!(init_fields.contains(&field.name))
            }

            type_ast::Expr::Struct {
                struct_ty,
                fields: type_fields,
            }
        }
        ast::Expr::Path { segments } => {
            let type_segments = analysis_path(semantic, segments)?;
            assert_eq!(segments.len(), 1);
            let segment = &segments[0];
            assert!(segment.args.is_empty());

            println!("{:?}", segments);
            let symbol = semantic.symbol_table.lookup(&segment.ident).unwrap();
            type_ast::Expr::Path {
                segments: type_segments,
                ty: symbol.ty().unwrap(),
                location: symbol.location,
            }
        }
    };

    Ok(type_expr)
}

fn analysis_path(
    semantic: &mut Semantic,
    segments: &[ast::PathSegment],
) -> Result<Vec<type_ast::PathSegment>, SemanticError> {
    let mut type_segments = Vec::new();

    for segment in segments {
        let mut type_args = Vec::new();

        for arg in &segment.args {
            let type_arg = semantic.type_id(&arg)?;
            type_args.push(type_arg);
        }

        let type_segment = type_ast::PathSegment {
            ident: segment.ident.to_string(),
            args: type_args,
        };

        type_segments.push(type_segment);
    }

    Ok(type_segments)
}

fn analysis_block(
    semantic: &mut Semantic,
    block: &ast::Block,
    return_ty: Option<TypeId>,
) -> Result<type_ast::Block, SemanticError> {
    let mut typed_block = type_ast::Block {
        statements: Vec::new(),
    };

    for stmt in &block.statements {
        println!("{:?}", stmt);
        let stmt = match stmt {
            ast::BlockStmt::Let(let_stmt) => {
                let expr = analysis_expr(semantic, &let_stmt.expr)?;

                let location = semantic
                    .symbol_table
                    .insert(&let_stmt.var_name, *expr.ty())?;
                type_ast::BlockStmt::Let(type_ast::LetStmt {
                    var_name: let_stmt.var_name.to_string(),
                    location,
                    expr,
                })
            }
            ast::BlockStmt::Assign(assign_stmt) => {
                type_ast::BlockStmt::Assign(type_ast::AssignStmt {
                    target: analysis_expr(semantic, &assign_stmt.target)?,
                    expr: analysis_expr(semantic, &assign_stmt.expr)?,
                })
            }
            ast::BlockStmt::Return(return_stmt) => {
                let expr = match &return_stmt.expr {
                    Some(expr) => Some(analysis_expr(semantic, &expr)?),
                    None => None,
                };

                assert_eq!(return_ty, expr.as_ref().map(|x| *x.ty()));
                type_ast::BlockStmt::Return(type_ast::ReturnStmt { expr })
            }
            ast::BlockStmt::Expr(expr) => {
                type_ast::BlockStmt::Expr(analysis_expr(semantic, &expr)?)
            }
            ast::BlockStmt::Block(block) => {
                semantic.symbol_table.enter_scope();
                let block = type_ast::BlockStmt::Block(Box::new(analysis_block(
                    semantic, block, return_ty,
                )?));
                semantic.symbol_table.exit_scope();
                block
            }
            ast::BlockStmt::If(if_stmt) => type_ast::BlockStmt::If(type_ast::IfStmt {
                condition: {
                    let condition = analysis_expr(semantic, &if_stmt.condition)?;
                    assert_eq!(*condition.ty(), TypeId::BOOL);
                    Box::new(condition)
                },
                then_branch: {
                    semantic.symbol_table.enter_scope();
                    let branch =
                        Box::new(analysis_block(semantic, &if_stmt.then_branch, return_ty)?);
                    semantic.symbol_table.exit_scope();
                    branch
                },
                else_branch: match &if_stmt.else_branch {
                    Some(else_branch) => {
                        semantic.symbol_table.enter_scope();
                        let branch =
                            Some(Box::new(analysis_block(semantic, else_branch, return_ty)?));

                        semantic.symbol_table.exit_scope();
                        branch
                    }
                    None => None,
                },
            }),
            ast::BlockStmt::While(while_stmt) => type_ast::BlockStmt::While(type_ast::WhileStmt {
                condition: {
                    let condition = analysis_expr(semantic, &while_stmt.condition)?;
                    assert_eq!(*condition.ty(), TypeId::BOOL);
                    Box::new(condition)
                },
                block: {
                    semantic.symbol_table.enter_scope();
                    let branch = Box::new(analysis_block(semantic, &while_stmt.block, return_ty)?);
                    semantic.symbol_table.exit_scope();
                    branch
                },
            }),
            ast::BlockStmt::Break => type_ast::BlockStmt::Break,
            ast::BlockStmt::Continue => type_ast::BlockStmt::Continue,
        };

        typed_block.statements.push(stmt);
    }

    Ok(typed_block)
}

fn analysis_func(
    semantic: &mut Semantic,
    fn_def: &ast::FunctionDef,
    this: Option<TypeId>,
) -> Result<type_ast::FunctionDef, SemanticError> {
    let mut type_args = Vec::new();
    semantic.symbol_table.enter_scope();

    if let Some(this) = this {
        let location = semantic.symbol_table.insert("self", this)?;

        let type_arg = type_ast::Arg {
            name: "self".to_string(),
            type_: this,
            location,
        };

        type_args.push(type_arg);
    }

    for arg in &fn_def.args {
        let type_ = semantic.type_id(&arg.type_)?;
        let location = semantic.symbol_table.insert(&arg.name, type_)?;
        let type_arg = type_ast::Arg {
            name: arg.name.to_string(),
            type_,
            location,
        };

        type_args.push(type_arg);
    }

    let return_type = match &fn_def.return_type {
        Some(ty) => Some(semantic.type_id(ty)?),
        None => None,
    };

    let typed_body = analysis_block(semantic, &fn_def.body, return_type)?;

    let local_count = semantic.symbol_table.take_max_local_count();
    semantic.symbol_table.exit_scope();

    let idx = semantic
            .symbol_table
            .lookup(&fn_def.name)
            .unwrap()
            .location
            .as_global()
            .unwrap();

    let typed_fn_def = type_ast::FunctionDef {
        name: fn_def.name.to_string(),
        args: type_args,
        local_count,
        return_type,
        body: typed_body,
        idx,
    };

    Ok(typed_fn_def)
}

#[derive(Debug)]
pub enum SemanticError {
    TypeMistmatch,
    DuplicateDef {
        name: String,
    },
    UndefinedIdent {
        name: String,
    },
    CanNotAssignTo {
        target: String,
    },
    UnknownType {
        name: String,
    },
    UnsupportBinaryOp {
        op: BinaryOp,
        left: TypeKind,
        right: TypeKind,
    },
    UnsupportUnaryOp {
        op: UnaryOp,
        ty: TypeKind,
    },
    MemberAssign,
    MissStructField {
        struct_name: String,
        field_name: String,
    },
    UnknownField {
        struct_name: String,
        field_name: String,
    },
    MissingEntryPoint,
    NonBooleanCondition,
    NonBooleanAnd,
    NonBooleanOr,
    InvalidBreak,
    InvalidContinue,
    DuplicateFieldInit {
        struct_name: String,
        field_name: String,
    },
    MissFieldInit {
        struct_name: String,
        field_name: String,
    },
    MissFunc {
        name: String,
    },
    EmptyTypeSegments,
    IndexTypeMismatch,
}

impl Error for SemanticError {}

impl Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(&self, f)
    }
}
