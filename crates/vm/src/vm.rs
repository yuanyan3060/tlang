use std::cmp::Ordering;
use std::error::Error;

use ast::Program;

use crate::bytecode::{ByteCode, Loc};
use crate::compiler::ConstValue;
use crate::mem::{Mem, Tracer};
use crate::package::{Function, Package};
use crate::value::{GcData, Value};

pub struct Vm {
    mem: Mem,
    local_offset: usize,
    temp_offset: usize,
    tracer: Tracer,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            mem: Mem::new(),
            local_offset: 0,
            temp_offset: 0,
            tracer: Tracer::new(),
        }
    }

    pub fn push_stack(&mut self, value: Value) {
        self.mem.push_stack(value);
    }

    pub fn add_const(&mut self, val: &ConstValue) {
        let val = match val {
            ConstValue::Nil => Value::Nil,
            ConstValue::Bool(b) => Value::Bool(*b),
            ConstValue::Int(i) => Value::Int(*i),
            ConstValue::Float(f) => Value::Float((*f).into()),
            ConstValue::String(s) => Value::String(self.mem.alloc(GcData::String(s.to_string()))),
        };
        self.mem.consts.push(val)
    }

    pub fn execute(&mut self, pkg: &Package) -> Result<(), Box<dyn Error>> {
        for val in &pkg.constants {
            self.add_const(val);
        }
        self.mem.global = pkg.global.to_vec();
        let main_fn = &pkg.functions[pkg.entry_function];

        let start = std::time::Instant::now();
        let ret = self.execute_fn(pkg, main_fn)?;
        println!("{:?}", start.elapsed());
        println!("{:?}", ret);
        Ok(())
    }

    pub fn execute_fn(&mut self, pkg: &Package, f: &Function) -> anyhow::Result<Value> {
        let mut ret = Value::Nil;

        match f {
            Function::Native { name, func } => todo!(),
            Function::Custom {
                name,
                codes,
                local_var_cnt,
                temp_var_cnt,
            } => {
                let before_local_offset = self.local_offset;
                let before_temp_offset = self.temp_offset;

                self.local_offset = self.mem.local.len();
                self.temp_offset = self.mem.temp.len();

                for _ in 0..*local_var_cnt {
                    self.mem.local.push(Value::Nil);
                }

                for _ in 0..*temp_var_cnt {
                    self.mem.temp.push(Value::Nil);
                }

                let mut idx = 0;
                while idx < codes.len() {
                    let code = &codes[idx];
                    idx += 1;

                    match code {
                        ByteCode::Pos { dst, src } => {
                            let src = self.get_var(*src);
                            self.set_var(*dst, src);
                        }
                        ByteCode::Neg { dst, src } => {
                            let src = self.get_var(*src);
                            let val = match src {
                                Value::Int(i) => Value::Int(-i),
                                Value::Float(f) => Value::Float(-f),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Not { dst, src } => {
                            let src = self.get_var(*src);
                            let val = match src {
                                Value::Bool(b) => Value::Bool(!b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::BitNot { dst, src } => {
                            let src = self.get_var(*src);
                            let val = match src {
                                Value::Int(i) => Value::Int(!i),
                                _ => unreachable!("BitNot expects integer"),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Add { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                                (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 + b),
                                (Value::Float(a), Value::Int(b)) => Value::Float(a + b as f64),
                                (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                                _ => unreachable!("Add only supports Int/Float"),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Sub { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                                (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 - b),
                                (Value::Float(a), Value::Int(b)) => Value::Float(a - b as f64),
                                (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Mul { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                                (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 * b),
                                (Value::Float(a), Value::Int(b)) => Value::Float(a * b as f64),
                                (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Div { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a / b), // 整数除法，注意 b != 0
                                (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 / b),
                                (Value::Float(a), Value::Int(b)) => Value::Float(a / b as f64),
                                (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Mod { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
                                _ => unreachable!("Mod only supports integers"),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::BitAnd { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a & b),
                                _ => unreachable!("BitAnd only supports integers"),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::BitOr { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a | b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::BitXor { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a ^ b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Shl { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a << b),
                                _ => unreachable!("Shl only supports integers"),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Shr { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a >> b),
                                _ => unreachable!("Shr only supports integers"),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Eq { dst, left, right } => {
                            let cmp = self.compare(*left, *right);
                            self.set_var(
                                *dst,
                                Value::Bool(cmp.map(|x| x.is_eq()).unwrap_or_default()),
                            );
                        }
                        ByteCode::Ne { dst, left, right } => {
                            let cmp = self.compare(*left, *right);
                            self.set_var(
                                *dst,
                                Value::Bool(cmp.map(|x| x.is_ne()).unwrap_or_default()),
                            );
                        }
                        ByteCode::Lt { dst, left, right } => {
                            let cmp = self.compare(*left, *right);
                            self.set_var(
                                *dst,
                                Value::Bool(cmp.map(|x| x.is_lt()).unwrap_or_default()),
                            );
                        }
                        ByteCode::Le { dst, left, right } => {
                            let cmp = self.compare(*left, *right);
                            self.set_var(
                                *dst,
                                Value::Bool(cmp.map(|x| x.is_le()).unwrap_or_default()),
                            );
                        }
                        ByteCode::Gt { dst, left, right } => {
                            let cmp = self.compare(*left, *right);
                            self.set_var(
                                *dst,
                                Value::Bool(cmp.map(|x| x.is_gt()).unwrap_or_default()),
                            );
                        }
                        ByteCode::Ge { dst, left, right } => {
                            let cmp = self.compare(*left, *right);
                            self.set_var(
                                *dst,
                                Value::Bool(cmp.map(|x| x.is_ge()).unwrap_or_default()),
                            );
                        }
                        ByteCode::And { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Bool(a), Value::Bool(b)) => Value::Bool(a && b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Or { dst, left, right } => {
                            let left = self.get_var(*left);
                            let right = self.get_var(*right);
                            let val = match (left, right) {
                                (Value::Bool(a), Value::Bool(b)) => Value::Bool(a || b),
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::GetParam { dst } => {
                            let val = self.mem.stack.pop().unwrap();
                            self.set_var(*dst, val);
                        }
                        ByteCode::Param { src } => {
                            let val = self.get_var(*src);
                            self.mem.stack.push(val);
                        }
                        ByteCode::NewObject { dst, size } => {
                            let val = vec![Value::Nil; *size as usize];
                            let val = GcData::Struct(val);
                            let val = self.mem.alloc(val);
                            let val = Value::Struct(val);
                            self.set_var(*dst, val);
                        }
                        ByteCode::Call {
                            dst,
                            func,
                            param_cnt,
                        } => {
                            let func = self.get_var(*func);
                            let val = match func {
                                Value::NativeFn(_) => todo!(),
                                Value::Fn(idx) => {
                                    let f = &pkg.functions[idx as usize];
                                    self.execute_fn(pkg, f)?
                                }
                                _ => unreachable!(),
                            };
                            self.set_var(*dst, val);
                        }
                        ByteCode::Load { from, to } => {
                            let from = self.get_var(*from);
                            self.set_var(*to, from);
                        }
                        ByteCode::JumpIfFalse { cond, offset } => {
                            let cond = self.get_var(*cond);
                            let cond = match cond {
                                Value::Bool(b) => b,
                                _ => unreachable!(),
                            };
                            if !cond {
                                idx = *offset as usize
                            }
                        }
                        ByteCode::Jump { offset } => idx = *offset as usize,
                        ByteCode::SetIndex { dst, idx, src } => {
                            let src = self.get_var(*src);
                            let dst = self.get_var(*dst);

                            let offset = match self.get_var(*idx) {
                                Value::Int(offset) => offset,
                                _ => unreachable!(),
                            };

                            let Value::Vec(dst) = dst else { unreachable!() };
                            let GcData::Vec(dst) = self.mem.get_mut(dst) else {
                                unreachable!()
                            };
                            dst[offset as usize] = src
                        }
                        ByteCode::Index { dst, idx, src } => {
                            let src = self.get_var(*src);

                            let offset = match self.get_var(*idx) {
                                Value::Int(offset) => offset,
                                _ => unreachable!(),
                            };
                            let Value::Vec(src) = src else { unreachable!() };
                            let GcData::Vec(src) = self.mem.get(src) else {
                                unreachable!()
                            };

                            let src = src[offset as usize];
                            self.set_var(*dst, src);
                        }
                        ByteCode::SetMember { dst, offset, src } => {
                            let src = self.get_var(*src);
                            let dst = self.get_var(*dst);
                            let Value::Struct(dst) = dst else {
                                unreachable!()
                            };
                            let GcData::Struct(dst) = self.mem.get_mut(dst) else {
                                unreachable!()
                            };
                            dst[*offset as usize] = src
                        }
                        ByteCode::Member { dst, offset, src } => {
                            let src = self.get_var(*src);
                            let Value::Struct(src) = src else {
                                unreachable!()
                            };
                            let GcData::Struct(src) = self.mem.get(src) else {
                                unreachable!()
                            };
                            let src = src[*offset as usize];
                            self.set_var(*dst, src);
                        }
                        ByteCode::Return { src } => {
                            if let Some(src) = src {
                                let val = self.get_var(*src);
                                ret = val;
                                self.mem.stack.push(val);
                            }
                            break;
                        }
                    }
                }

                self.mem.local.truncate(self.local_offset);
                self.mem.temp.truncate(self.temp_offset);

                self.local_offset = before_local_offset;
                self.temp_offset = before_temp_offset;
            }
        }
        Ok(ret)
    }

    #[inline(always)]
    pub(crate) fn set_var(&mut self, var: Loc, value: Value) {
        let tag = var.0 >> 14;
        let idx = (var.0 & 0x3fff) as usize;

        unsafe {
            match tag {
                0 => *self.mem.local.get_unchecked_mut(idx + self.local_offset) = value,
                1 => *self.mem.temp.get_unchecked_mut(idx + self.temp_offset) = value,
                2 => *self.mem.global.get_unchecked_mut(idx) = value,
                3 => *self.mem.consts.get_unchecked_mut(idx) = value,
                _ => unreachable!(),
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get_var(&self, var: Loc) -> Value {
        let tag = var.0 >> 14;
        let idx = (var.0 & 0x3fff) as usize;
        unsafe {
            match tag {
                0 => *self.mem.local.get_unchecked(idx + self.local_offset),
                1 => *self.mem.temp.get_unchecked(idx + self.temp_offset),
                2 => *self.mem.global.get_unchecked(idx),
                3 => *self.mem.consts.get_unchecked(idx),
                _ => unreachable!(),
            }
        }
    }

    #[inline(always)]
    fn compare(&self, left: Loc, right: Loc) -> Option<Ordering> {
        let left = self.get_var(left);
        let right = self.get_var(right);

        match (left, right) {
            (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            (Value::Bool(left), Value::Bool(right)) => Some(left.cmp(&right)),
            (Value::Int(left), Value::Int(right)) => Some(left.cmp(&right)),
            (Value::Float(left), Value::Float(right)) => left.partial_cmp(&right),
            (Value::String(left), Value::String(right)) => {
                if left == right {
                    return Some(Ordering::Equal);
                }

                let left = self.mem.get(left);
                let right = self.mem.get(right);

                match (left, right) {
                    (GcData::String(left), GcData::String(right)) => Some(left.cmp(right)),
                    _ => None,
                }
            }
            (Value::Struct(left), Value::Struct(right)) => Some(left.cmp(&right)),
            (Value::Vec(left), Value::Vec(right)) => Some(left.cmp(&right)),
            (Value::NativeFn(_), Value::NativeFn(_)) => None,
            (Value::Fn(left), Value::Fn(right)) => Some(left.cmp(&right)),
            _ => None,
        }
    }
}

pub struct CallFrame {}
