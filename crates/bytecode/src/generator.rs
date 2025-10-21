use ast::{BinaryOp, UnaryOp};
use std::collections::HashMap;
use std::fmt::Display;
use value::{NativeFnPtr, Type};

use crate::ByteCode;

pub struct Generator {
    pub constant_map: Map<String>,
    pub struct_map: Map<StructType>,
    pub fn_map: Map<FnType>,
    pub functions: Vec<Function>,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    pub fn new() -> Self {
        Self {
            constant_map: Map::new(),
            struct_map: Map::new(),
            fn_map: Map::new(),
            functions: Vec::new(),
        }
    }

    pub fn add_const(&mut self, text: &str) -> usize {
        if let Some(idx) = self.constant_map.get_idx(text) {
            return idx;
        }
        self.constant_map.insert(text, text.to_string())
    }

    pub fn register_native_fn(
        &mut self,
        name: &str,
        func: NativeFnPtr,
        args: Vec<Type>,
        return_type: Type,
    ) -> Result<()> {
        if self.fn_map.contains_key(name) {
            return Err(Error::DuplicateDef {
                name: name.to_string(),
            });
        }

        self.fn_map.insert(
            name,
            FnType {
                args: args.clone(),
                return_ty: Some(return_type),
                self_ty: None,
            },
        );

        self.functions.push(Function::Native {
            name: name.to_string(),
            func,
            return_type: Some(return_type),
        });
        Ok(())
    }

    pub fn build_struct_map(&mut self, program: &ast::Program) -> Result<()> {
        for stmt in &program.statements {
            if let ast::Statement::StructDef(struct_def) = stmt {
                let name = &struct_def.name;
                if self.struct_map.contains_key(name) {
                    continue;
                }
                let val = StructType {
                    name: name.to_string(),
                    fields: Vec::new(),
                };
                self.struct_map.insert(name, val);
            }
        }

        for stmt in &program.statements {
            if let ast::Statement::StructDef(struct_def) = stmt {
                let name = &struct_def.name;
                let mut fields = Vec::new();
                for f in &struct_def.fields {
                    let ty = self.get_type(&f.type_.name)?;
                    fields.push(Field {
                        name: f.name.to_string(),
                        ty,
                    });
                }
                self.struct_map.get_name_mut(name).unwrap().fields = fields;
            }
        }
        Ok(())
    }

    pub fn build_func_map(&mut self, program: &ast::Program) -> Result<()> {
        for stmt in &program.statements {
            match stmt {
                ast::Statement::FunctionDef(fn_def) => {
                    let name = &fn_def.name;
                    if self.fn_map.contains_key(name) {
                        return Err(Error::DuplicateDef {
                            name: name.to_string(),
                        });
                    }

                    let mut args = Vec::new();
                    for i in &fn_def.args {
                        let arg = self.get_type(&i.type_.name)?;
                        args.push(arg);
                    }

                    let return_ty = match &fn_def.return_type {
                        Some(ty) => Some(self.get_type(&ty.name)?),
                        None => None,
                    };

                    let val = FnType {
                        args,
                        return_ty,
                        self_ty: None,
                    };
                    self.fn_map.insert(name, val);
                }
                ast::Statement::StructDef(struct_def) => {
                    let self_ty = self.get_type(&struct_def.name)?;
                    for fn_def in &struct_def.functions {
                        let name = &fn_def.name;
                        if self.fn_map.contains_key(name) {
                            return Err(Error::DuplicateDef {
                                name: name.to_string(),
                            });
                        }

                        let mut args = Vec::new();
                        for i in &fn_def.args {
                            let arg = self.get_type(&i.type_.name)?;
                            args.push(arg);
                        }

                        let return_ty = match &fn_def.return_type {
                            Some(ty) => Some(self.get_type(&ty.name)?),
                            None => None,
                        };

                        let val = FnType {
                            args,
                            return_ty,
                            self_ty: None,
                        };
                        self.fn_map.insert(name, val);
                    }

                    for fn_def in &struct_def.methods {
                        let name = &fn_def.name;
                        if self.fn_map.contains_key(name) {
                            return Err(Error::DuplicateDef {
                                name: name.to_string(),
                            });
                        }

                        let mut args = Vec::new();
                        for i in &fn_def.args {
                            let arg = self.get_type(&i.type_.name)?;
                            args.push(arg);
                        }

                        let return_ty = match &fn_def.return_type {
                            Some(ty) => Some(self.get_type(&ty.name)?),
                            None => None,
                        };

                        let val = FnType {
                            args,
                            return_ty,
                            self_ty: Some(self_ty),
                        };
                        let name = format!("{}::{}", struct_def.name, name);
                        self.fn_map.insert(&name, val);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_type(&mut self, name: &str) -> Result<Type> {
        match name {
            "nil" => Ok(Type::Nil),
            "bool" => Ok(Type::Bool),
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "str" => Ok(Type::String),
            _ => {
                if let Some(idx) = self.struct_map.get_idx(name) {
                    Ok(Type::Struct(idx as u32))
                } else if let Some(idx) = self.fn_map.get_idx(name) {
                    Ok(Type::Func(idx as u32))
                } else {
                    Err(Error::UnknownType {
                        name: name.to_string(),
                    })
                }
            }
        }
    }

    pub fn get_member(&self, struct_idx: usize, member: &str) -> Result<Member> {
        let struct_def = self.struct_map.get(struct_idx).unwrap();
        for (i, field) in struct_def.fields.iter().enumerate() {
            if field.name == member {
                return Ok(Member::Field {
                    offset: i,
                    ty: field.ty,
                });
            }
        }

        if let Some((idx, f)) = self
            .fn_map
            .get_full(&format!("{}::{}", struct_def.name, member))
        {
            return Ok(Member::Method {
                idx,
                return_ty: f.return_ty.unwrap_or(Type::Nil),
            });
        };
        Err(Error::MissStructField {
            struct_name: struct_def.name.to_string(),
            field_name: member.to_string(),
        })
    }

    pub fn compile(&mut self, program: &ast::Program) -> Result<Program> {
        let mut functions = std::mem::take(&mut self.functions);
        self.build_struct_map(program)?;
        self.build_func_map(program)?;

        let mut entry_function = None;

        for stmt in &program.statements {
            match stmt {
                ast::Statement::StructDef(struct_def) => {
                    let self_ty = self.get_type(&struct_def.name)?;
                    for function_def in &struct_def.methods {
                        let return_ty = function_def
                            .return_type
                            .as_ref()
                            .map(|x| self.get_type(&x.name))
                            .transpose()?;
                        let name = format!("{}::{}", struct_def.name, function_def.name);
                        let f = self.compile_fn(
                            &name,
                            &function_def.args,
                            &function_def.body,
                            Some(self_ty),
                            return_ty,
                            function_def.name == "new",
                        )?;

                        functions.push(f);
                    }
                }
                ast::Statement::FunctionDef(function_def) => {
                    let return_ty = function_def
                        .return_type
                        .as_ref()
                        .map(|x| self.get_type(&x.name))
                        .transpose()?;
                    let f = self.compile_fn(
                        &function_def.name,
                        &function_def.args,
                        &function_def.body,
                        None,
                        return_ty,
                        false,
                    )?;
                    if function_def.name == "main" {
                        entry_function = Some(functions.len())
                    }
                    functions.push(f);
                }
            }
        }

        let entry_function = entry_function.ok_or(Error::MissingEntryPoint)?;
        Ok(Program {
            constants: self.constant_map.data.clone(),
            structs: self.struct_map.data.clone(),
            functions,
            entry_function,
        })
    }

    pub fn compile_block(
        &mut self,
        body: &ast::Block,
        local_vars: &mut HashMap<String, LocalVar>,
        codes: &mut Vec<ByteCode>,
        is_init: bool,
    ) -> Result<JumpIndex> {
        let mut jumps = JumpIndex::default();

        for stmt in &body.statements {
            match stmt {
                ast::BlockStmt::Let(let_stmt) => {
                    if local_vars.contains_key(let_stmt.var_name.as_str()) {
                        return Err(Error::DuplicateDef {
                            name: let_stmt.var_name.to_string(),
                        });
                    }
                    let idx = local_vars.len();
                    let ty = self.compile_expr(local_vars, &let_stmt.expr, codes)?;
                    local_vars.insert(
                        let_stmt.var_name.to_string(),
                        LocalVar {
                            idx: idx as u32,
                            ty,
                        },
                    );
                    codes.push(ByteCode::Store { idx: idx as u32 });
                }
                ast::BlockStmt::Assign(assign_stmt) => match &assign_stmt.target {
                    ast::Expr::Ident(name) => {
                        let Some(var) = local_vars.get(name.as_str()) else {
                            return Err(Error::UndefinedIdent {
                                name: name.to_string(),
                            });
                        };
                        self.compile_expr(local_vars, &assign_stmt.expr, codes)?;
                        codes.push(ByteCode::Store { idx: var.idx });
                    }
                    ast::Expr::Index { .. } => todo!(),
                    ast::Expr::Member { target, member } => {
                        let target_ty = self.compile_expr(local_vars, target, codes)?;
                        match target_ty {
                            Type::Struct(idx) => match self.get_member(idx as usize, member)? {
                                Member::Field { offset, .. } => {
                                    self.compile_expr(local_vars, &assign_stmt.expr, codes)?;
                                    codes.push(ByteCode::SetField {
                                        offset: offset as u32,
                                    });
                                }
                                Member::Method { .. } => return Err(Error::MemberAssign),
                            },
                            _ => return Err(Error::MemberAssign),
                        };
                        self.compile_expr(local_vars, &assign_stmt.expr, codes)?;
                    }
                    ast::Expr::Literal(_) => {
                        return Err(Error::CanNotAssignTo {
                            target: "literal".to_string(),
                        });
                    }
                    ast::Expr::Unary { .. } => {
                        return Err(Error::CanNotAssignTo {
                            target: "unary expr".to_string(),
                        });
                    }
                    ast::Expr::Binary { .. } => {
                        return Err(Error::CanNotAssignTo {
                            target: "binary expr".to_string(),
                        });
                    }
                    ast::Expr::Call { .. } => {
                        return Err(Error::CanNotAssignTo {
                            target: "func call".to_string(),
                        });
                    }
                },
                ast::BlockStmt::Return(return_stmt) => {
                    if let Some(expr) = &return_stmt.expr {
                        self.compile_expr(local_vars, expr, codes)?;
                    } else if is_init {
                        codes.push(ByteCode::Load { idx: 0 });
                    } else {
                        codes.push(ByteCode::LoadNil);
                    }
                    codes.push(ByteCode::Return);
                }
                ast::BlockStmt::Expr(expr) => {
                    self.compile_expr(local_vars, expr, codes)?;
                    codes.push(ByteCode::Pop);
                }
                ast::BlockStmt::If(if_stmt) => {
                    let condition = self.compile_expr(local_vars, &if_stmt.condition, codes)?;
                    if condition != Type::Bool {
                        return Err(Error::NonBooleanCondition);
                    }
                    let then_start = codes.len();
                    codes.push(ByteCode::JumpIfFalse { offset: 0 });
                    let jump =
                        self.compile_block(&if_stmt.then_branch, local_vars, codes, is_init)?;
                    jumps.extend(jump);

                    let else_start = codes.len();
                    codes.push(ByteCode::Jump { offset: 0 });
                    if let Some(block) = &if_stmt.else_branch {
                        let jump = self.compile_block(block, local_vars, codes, is_init)?;
                        jumps.extend(jump);
                    }
                    let else_end = codes.len();
                    codes[then_start] = ByteCode::JumpIfFalse {
                        offset: else_start as u32 + 1,
                    };
                    codes[else_start] = ByteCode::Jump {
                        offset: else_end as u32,
                    };
                }
                ast::BlockStmt::While(while_stmt) => {
                    let while_start = codes.len();
                    let condition = self.compile_expr(local_vars, &while_stmt.condition, codes)?;
                    if condition != Type::Bool {
                        return Err(Error::NonBooleanCondition);
                    }
                    let start = codes.len();
                    codes.push(ByteCode::JumpIfFalse { offset: 0 });

                    let jump = self.compile_block(&while_stmt.block, local_vars, codes, is_init)?;

                    codes.push(ByteCode::Jump {
                        offset: while_start as u32,
                    });
                    let offset = codes.len();
                    codes[start] = ByteCode::JumpIfFalse {
                        offset: offset as u32,
                    };

                    for b in jump.breaks {
                        codes[b] = ByteCode::Jump {
                            offset: offset as u32,
                        };
                    }

                    for c in jump.continues {
                        codes[c] = ByteCode::Jump {
                            offset: while_start as u32,
                        };
                    }
                }
                ast::BlockStmt::Continue => {
                    jumps.continues.push(codes.len());
                    codes.push(ByteCode::Jump { offset: u32::MAX });
                }
                ast::BlockStmt::Break => {
                    jumps.breaks.push(codes.len());
                    codes.push(ByteCode::Jump { offset: u32::MAX });
                }
                ast::BlockStmt::Block(block) => {
                    let block_jump = self.compile_block(block, local_vars, codes, is_init)?;
                    jumps.extend(block_jump);
                }
            }
        }
        Ok(jumps)
    }

    pub fn compile_fn(
        &mut self,
        name: &str,
        args: &[ast::Arg],
        body: &ast::Block,
        self_type: Option<Type>,
        return_type: Option<Type>,
        is_init: bool,
    ) -> Result<Function> {
        let mut local_vars = HashMap::new();

        if let Some(ty) = self_type {
            let idx = local_vars.len();
            local_vars.insert(
                "self".to_string(),
                LocalVar {
                    idx: idx as u32,
                    ty,
                },
            );
        }

        for arg in args {
            let idx = local_vars.len() as u32;
            let ty = self.get_type(&arg.type_.name)?;
            local_vars.insert(arg.name.to_string(), LocalVar { idx, ty });
        }

        let mut codes = Vec::new();
        for i in (0..local_vars.len()).rev() {
            codes.push(ByteCode::Store { idx: i as u32 });
        }
        codes.push(ByteCode::Pop);

        let jump = self.compile_block(body, &mut local_vars, &mut codes, is_init)?;
        if !jump.breaks.is_empty() {
            return Err(Error::InvalidBreak);
        }

        if !jump.continues.is_empty() {
            return Err(Error::InvalidContinue);
        }

        if is_init {
            codes.push(ByteCode::Load { idx: 0 });
        } else {
            codes.push(ByteCode::LoadNil);
        }

        codes.push(ByteCode::Return);
        Ok(Function::Custom {
            name: name.to_string(),
            codes,
            local_var_cnt: local_vars.len() as u32,
            return_type,
        })
    }

    pub fn compile_expr(
        &mut self,
        local_vars: &HashMap<String, LocalVar>,
        expr: &ast::Expr,
        codes: &mut Vec<ByteCode>,
    ) -> Result<Type> {
        match expr {
            ast::Expr::Ident(name) => {
                // 先找局部变量
                if let Some(var) = local_vars.get(name.as_str()) {
                    codes.push(ByteCode::Load { idx: var.idx });
                    return Ok(var.ty);
                }

                // 然后找函数表
                if let Some((idx, _)) = self.fn_map.get_full(name) {
                    codes.push(ByteCode::LoadFunction { idx: idx as u32 });
                    return Ok(Type::Func(idx as u32));
                }

                // 最后找 struct
                if let Some((idx, st)) = self.struct_map.get_full(name) {
                    let f_name = format!("{}::new", name);
                    let Some((f_idx, _)) = self.fn_map.get_full(&f_name) else {
                        return Err(Error::UndefinedIdent { name: f_name });
                    };
                    codes.push(ByteCode::LoadFunction { idx: f_idx as u32 });
                    codes.push(ByteCode::NewStruct {
                        idx: idx as u32,
                        cnt: st.fields.len() as _,
                    });
                    return Ok(Type::Struct(idx as u32));
                }

                Err(Error::UndefinedIdent {
                    name: name.to_string(),
                })
            }
            ast::Expr::Literal(literal) => match literal {
                token::Literal::Nil => {
                    codes.push(ByteCode::LoadNil);
                    Ok(Type::Nil)
                }
                token::Literal::Bool(v) => {
                    codes.push(ByteCode::LoadBool { val: *v });
                    Ok(Type::Bool)
                }
                token::Literal::Int(v) => {
                    codes.push(ByteCode::LoadInt { val: *v });
                    Ok(Type::Int)
                }
                token::Literal::Float(v) => {
                    codes.push(ByteCode::LoadFloat { val: *v });
                    Ok(Type::Float)
                }
                token::Literal::String(v) => {
                    let idx = self.add_const(v);
                    codes.push(ByteCode::LoadString { idx: idx as u32 });
                    Ok(Type::String)
                }
            },
            ast::Expr::Unary { op, expr } => {
                let ty = self.compile_expr(local_vars, expr, codes)?;
                match op {
                    UnaryOp::Plus => match ty {
                        Type::Int => Ok(Type::Int),
                        Type::Float => Ok(Type::Float),
                        _ => Err(Error::UnsupportUnaryOp { op: *op, ty }),
                    },
                    UnaryOp::Minus => {
                        codes.push(ByteCode::Minus);
                        match ty {
                            Type::Int => Ok(Type::Int),
                            Type::Float => Ok(Type::Float),
                            _ => Err(Error::UnsupportUnaryOp { op: *op, ty }),
                        }
                    }
                    UnaryOp::Not => {
                        codes.push(ByteCode::Not);
                        match ty {
                            Type::Bool => Ok(Type::Bool),
                            _ => Err(Error::UnsupportUnaryOp { op: *op, ty }),
                        }
                    }
                    UnaryOp::BitNot => {
                        codes.push(ByteCode::BitNot);
                        match ty {
                            Type::Int => Ok(Type::Int),
                            _ => Err(Error::UnsupportUnaryOp { op: *op, ty }),
                        }
                    }
                }
            }
            ast::Expr::Binary { left, op, right } => {
                let left = self.compile_expr(local_vars, left, codes)?;
                let right = self.compile_expr(local_vars, right, codes)?;

                match op {
                    BinaryOp::Add => codes.push(ByteCode::Add),
                    BinaryOp::Subtract => {
                        codes.push(ByteCode::Subtract);
                    }
                    BinaryOp::Multiply => codes.push(ByteCode::Multiply),
                    BinaryOp::Divide => codes.push(ByteCode::Divide),
                    BinaryOp::Modulo => codes.push(ByteCode::Modulo),
                    BinaryOp::BitAnd => {
                        if (left, right) != (Type::Int, Type::Int) {
                            return Err(Error::UnsupportBinaryOp {
                                op: *op,
                                left,
                                right,
                            });
                        }
                        codes.push(ByteCode::BitAnd);
                    }
                    BinaryOp::BitOr => {
                        if (left, right) != (Type::Int, Type::Int) {
                            return Err(Error::UnsupportBinaryOp {
                                op: *op,
                                left,
                                right,
                            });
                        }
                        codes.push(ByteCode::BitOr);
                    }
                    BinaryOp::BitXor => {
                        if (left, right) != (Type::Int, Type::Int) {
                            return Err(Error::UnsupportBinaryOp {
                                op: *op,
                                left,
                                right,
                            });
                        }
                        codes.push(ByteCode::BitXor);
                    }
                    BinaryOp::ShiftLeft => todo!(),
                    BinaryOp::ShiftRight => todo!(),
                    BinaryOp::Equal => {
                        codes.push(ByteCode::Equal);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::NotEqual => {
                        codes.push(ByteCode::NotEqual);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::Less => {
                        codes.push(ByteCode::Less);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::LessEqual => {
                        codes.push(ByteCode::LessEqual);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::Greater => {
                        codes.push(ByteCode::Greater);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::GreaterEqual => {
                        codes.push(ByteCode::GreaterEqual);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::And => {
                        if (left, right) != (Type::Bool, Type::Bool) {
                            return Err(Error::NonBooleanAnd);
                        }
                        codes.push(ByteCode::And);
                        return Ok(Type::Bool);
                    }
                    BinaryOp::Or => {
                        if (left, right) != (Type::Bool, Type::Bool) {
                            return Err(Error::NonBooleanOr);
                        }
                        codes.push(ByteCode::Or);
                        return Ok(Type::Bool);
                    }
                }

                match (left, right) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Float, Type::Float) => Ok(Type::Float),
                    (Type::String, Type::String) => Ok(Type::String),
                    _ => Err(Error::UnsupportBinaryOp {
                        op: *op,
                        left,
                        right,
                    }),
                }
            }
            ast::Expr::Call { func, args } => {
                let mut is_method = false;
                let return_type = match &**func {
                    ast::Expr::Member { target, member } => {
                        let target_ty = self.compile_expr(local_vars, target, codes)?;
                        match target_ty {
                            Type::Struct(idx) => match self.get_member(idx as usize, member)? {
                                Member::Field { offset, ty } => {
                                    codes.push(ByteCode::GetField {
                                        offset: offset as u32,
                                    });
                                    ty
                                }
                                Member::Method { idx, return_ty } => {
                                    is_method = true;
                                    codes.push(ByteCode::LoadFunction { idx: idx as u32 });
                                    codes.push(ByteCode::Swap);
                                    return_ty
                                }
                            },
                            _ => return Err(Error::MemberAssign),
                        }
                    }
                    _ => {
                        let fn_type = self.compile_expr(local_vars, func, codes)?;
                        match fn_type {
                            Type::Nil => todo!(),
                            Type::Bool => todo!(),
                            Type::Int => todo!(),
                            Type::Float => todo!(),
                            Type::String => todo!(),
                            Type::Struct(idx) => {
                                is_method = true;
                                Type::Struct(idx)
                            }
                            Type::Func(idx) => {
                                let f = self.fn_map.get(idx as usize).unwrap();
                                f.return_ty.unwrap_or(Type::Nil)
                            }
                        }
                    }
                };
                for arg in args {
                    self.compile_expr(local_vars, arg, codes)?;
                }
                codes.push(ByteCode::Call {
                    arg_cnt: args.len() as u16 + is_method as u16,
                });
                Ok(return_type)
            }
            ast::Expr::Index { target, index } => {
                self.compile_expr(local_vars, target, codes)?;
                self.compile_expr(local_vars, index, codes)?;
                codes.push(ByteCode::GetIndex);
                todo!()
            }
            ast::Expr::Member { target, member } => {
                let target_ty = self.compile_expr(local_vars, target, codes)?;
                match target_ty {
                    Type::Struct(idx) => match self.get_member(idx as usize, member)? {
                        Member::Field { offset, ty } => {
                            codes.push(ByteCode::GetField {
                                offset: offset as u32,
                            });
                            Ok(ty)
                        }
                        Member::Method { .. } => unreachable!(),
                    },
                    _ => Err(Error::MemberAssign),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

pub struct FnType {
    pub args: Vec<Type>,
    pub return_ty: Option<Type>,
    pub self_ty: Option<Type>,
}

#[derive(Debug)]
pub enum Function {
    Native {
        name: String,
        func: NativeFnPtr,
        return_type: Option<Type>,
    },
    Custom {
        name: String,
        codes: Vec<ByteCode>,
        local_var_cnt: u32,
        return_type: Option<Type>,
    },
}

impl Function {
    pub fn name(&self) -> &str {
        match self {
            Function::Native { name, .. } => name,
            Function::Custom { name, .. } => name,
        }
    }

    pub fn return_type(&self) -> Option<Type> {
        match self {
            Function::Native {
                return_type: return_type_id,
                ..
            } => *return_type_id,
            Function::Custom {
                return_type: return_type_id,
                ..
            } => *return_type_id,
        }
    }

    pub fn local_var_cnt(&self) -> usize {
        match self {
            Function::Native { .. } => 0,
            Function::Custom { local_var_cnt, .. } => *local_var_cnt as usize,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub constants: Vec<String>,
    pub structs: Vec<StructType>,
    pub functions: Vec<Function>,
    pub entry_function: usize,
}

#[derive(Debug)]
pub enum Error {
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
        left: Type,
        right: Type,
    },
    UnsupportUnaryOp {
        op: UnaryOp,
        ty: Type,
    },
    MemberAssign,
    MissStructField {
        struct_name: String,
        field_name: String,
    },
    MissingEntryPoint,
    NonBooleanCondition,
    NonBooleanAnd,
    NonBooleanOr,
    InvalidBreak,
    InvalidContinue,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct LocalVar {
    pub idx: u32,
    pub ty: Type,
}

pub struct Map<T> {
    pub data: Vec<T>,
    pub names: HashMap<String, usize>,
}

impl<T> Default for Map<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Map<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            names: HashMap::new(),
        }
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    pub fn insert(&mut self, name: &str, value: T) -> usize {
        if let Some(id) = self.get_idx(name) {
            return id;
        }

        let idx = self.data.len();
        self.data.push(value);
        self.names.insert(name.to_string(), idx);
        idx
    }

    pub fn get_name(&self, name: &str) -> Option<&T> {
        let idx = self.names.get(name)?;
        self.data.get(*idx)
    }

    pub fn get_name_mut(&mut self, name: &str) -> Option<&mut T> {
        let idx = self.names.get(name)?;
        self.data.get_mut(*idx)
    }

    pub fn get_idx(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub fn get_full(&self, name: &str) -> Option<(usize, &T)> {
        let idx = self.get_idx(name)?;
        Some((idx, self.data.get(idx)?))
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.data.get(idx)
    }
}

pub enum Member {
    Field { offset: usize, ty: Type },
    Method { idx: usize, return_ty: Type },
}

#[derive(Default)]
pub struct JumpIndex {
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

impl JumpIndex {
    pub fn extend(&mut self, other: JumpIndex) {
        self.breaks.extend(other.breaks);
        self.continues.extend(other.continues);
    }
}
