use std::{error::Error, path::Path, rc::Rc};

use bytecode::{
    ByteCode,
    generator::{Function, Generator, Program},
};
use lex::Lex;
use parser::Parser;

use value::{Object, State, Type, Value};

pub mod builtin;

pub struct Vm {
    pub state: State,
    pub program: Option<Program>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            program: None,
        }
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let code = std::fs::read_to_string(path)?;
        let lex = Lex::new(code.chars());
        let tokens = lex.all();
        #[cfg(debug_assertions)]
        lex::pretty_print(&tokens);
        let mut parser = Parser::new(tokens.iter());
        let program = parser.parse_program()?;
        let mut g = Generator::new();
        g.register_native_fn(
            "print",
            builtin::builtin_print,
            vec![Type::Void],
            Type::Void,
        )?;
        g.register_native_fn("timestamp", builtin::builtin_timestamp, vec![], Type::Float)?;
        let p = g.compile(&program)?;
        #[cfg(debug_assertions)]
        println!("{:#?}", p);
        self.execute(&p)
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), Box<dyn Error>> {
        let main_fn = &program.functions[program.entry_function];
        #[cfg(debug_assertions)]
        {
            match main_fn {
                Function::Native { .. } => {}
                Function::Custom { codes, .. } => {
                    for (i, code) in codes.iter().enumerate() {
                        println!("{}: {:?}", i, code);
                    }
                }
            }
        }
        self.execute_codes(program, main_fn)
    }

    pub fn execute_codes(&mut self, program: &Program, f: &Function) -> Result<(), Box<dyn Error>> {
        match f {
            Function::Native { func, .. } => {
                let val = func(&mut self.state);
                self.state.stack.push(val);
            }
            Function::Custom {
                codes,
                local_var_cnt,
                ..
            } => {
                let local_offset = self.state.locals.len();
                for _ in 0..*local_var_cnt {
                    self.state.locals.push(Value::Void);
                }

                let mut code_offset = 0;
                while code_offset < codes.len() {
                    let locals = &mut self.state.locals[local_offset..];
                    let code = &codes[code_offset];
                    #[cfg(debug_assertions)]
                    println!("{:?} {:?} {:?}", code, self.state.stack, locals);
                    match code {
                        ByteCode::Pop => {
                            self.state.stack.pop();
                        }
                        ByteCode::LoadNil => {
                            self.state.stack.push(Value::Nil);
                        }
                        ByteCode::LoadBool { val } => {
                            self.state.stack.push(Value::Bool(*val));
                        }
                        ByteCode::LoadInt { val } => {
                            self.state.stack.push(Value::Int(*val));
                        }
                        ByteCode::LoadFloat { val } => {
                            self.state.stack.push(Value::Float(*val));
                        }
                        ByteCode::LoadString { idx } => {
                            let val = program.constants[*idx as usize].clone();
                            self.state.stack.push(val);
                        }
                        ByteCode::LoadFunction { idx } => {
                            self.state.stack.push(Value::Fn(*idx));
                        }
                        ByteCode::NewStruct { idx, cnt } => {
                            self.state.stack.push(Value::Object(Object::new(
                                Type::Struct(*idx),
                                *cnt as usize,
                            )));
                        }
                        ByteCode::Store { idx } => {
                            let val = self.state.stack.pop().unwrap();
                            locals[*idx as usize] = val;
                        }
                        ByteCode::Load { idx } => {
                            self.state.stack.push(locals[*idx as usize].clone());
                        }
                        ByteCode::GetField { offset } => {
                            let mut obj = self.state.stack.pop().unwrap();
                            let field =
                                obj.as_obj().unwrap().fields.borrow()[*offset as usize].clone();
                            self.state.stack.push(field);
                        }
                        ByteCode::SetField { offset } => {
                            let val = self.state.stack.pop().unwrap();
                            let mut target = self.state.stack.pop().unwrap();
                            target.as_obj().unwrap().fields.borrow_mut()[*offset as usize] = val;
                        }
                        ByteCode::Add => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l + r),
                                (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
                                (Value::String(l), Value::String(r)) => {
                                    Value::String(Rc::new(format!("{}{}", r, l)))
                                }
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Subtract => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l - r),
                                (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
                                (Value::String(l), Value::String(r)) => {
                                    Value::String(Rc::new(format!("{}{}", r, l)))
                                }
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Multiply => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l * r),
                                (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Divide => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l / r),
                                (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Modulo => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l % r),
                                (Value::Float(l), Value::Float(r)) => Value::Float(l % r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::BitAnd => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l & r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::BitOr => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l | r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::BitXor => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l ^ r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Equal => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Bool(l == r),
                                (Value::Float(l), Value::Float(r)) => Value::Bool(l == r),
                                (Value::String(l), Value::String(r)) => Value::Bool(l == r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::NotEqual => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Bool(l != r),
                                (Value::Float(l), Value::Float(r)) => Value::Bool(l != r),
                                (Value::String(l), Value::String(r)) => Value::Bool(l != r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Less => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Bool(l < r),
                                (Value::Float(l), Value::Float(r)) => Value::Bool(l < r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Greater => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Bool(l > r),
                                (Value::Float(l), Value::Float(r)) => Value::Bool(l > r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::LessEqual => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Bool(l <= r),
                                (Value::Float(l), Value::Float(r)) => Value::Bool(l <= r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::GreaterEqual => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Bool(l >= r),
                                (Value::Float(l), Value::Float(r)) => Value::Bool(l >= r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::And => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Bool(l), Value::Bool(r)) => Value::Bool(l && r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Or => {
                            let right = self.state.stack.pop().unwrap();
                            let left = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Bool(l), Value::Bool(r)) => Value::Bool(l || r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
                        }
                        ByteCode::Minus => {
                            let val = self.state.stack.pop().unwrap();
                            match val {
                                Value::Int(v) => {
                                    self.state.stack.push(Value::Int(-v));
                                }
                                Value::Float(v) => {
                                    self.state.stack.push(Value::Float(-v));
                                }
                                _ => unreachable!(),
                            }
                        }
                        ByteCode::Not => {
                            let val = self.state.stack.pop().unwrap();
                            match val {
                                Value::Bool(v) => {
                                    self.state.stack.push(Value::Bool(!v));
                                }
                                _ => unreachable!(),
                            }
                        }
                        ByteCode::BitNot => {
                            let val = self.state.stack.pop().unwrap();
                            match val {
                                Value::Int(v) => {
                                    self.state.stack.push(Value::Int(!v));
                                }
                                _ => unreachable!(),
                            }
                        }
                        ByteCode::GetIndex => todo!(),
                        ByteCode::Call { param_cnt } => {
                            let idx = self.state.stack.len() - *param_cnt as usize - 1;
                            if let Some(f) = self.state.stack[idx].as_fn() {
                                let f = &program.functions[f as usize];
                                self.execute_codes(program, f)?;
                            }
                        }
                        ByteCode::Swap => {
                            let i = self.state.stack.len() - 1;
                            let j = i - 1;
                            self.state.stack.swap(i, j);
                        }
                        ByteCode::Return => {
                            break;
                        }
                        ByteCode::JumpIfFalse { offset } => {
                            let val = self.state.stack.pop().unwrap().as_bool().unwrap();
                            if !val {
                                code_offset = *offset as usize;
                                continue;
                            }
                        }
                        ByteCode::Jump { offset } => {
                            code_offset = *offset as usize;
                            continue;
                        }
                        ByteCode::Nop => {}
                    }

                    code_offset += 1;
                }
                for _ in 0..*local_var_cnt {
                    self.state.locals.pop();
                }
            }
        };

        Ok(())
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
