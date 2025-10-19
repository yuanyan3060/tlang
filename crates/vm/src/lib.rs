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
        //lex::pretty_print(&tokens);
        let mut parser = Parser::new(tokens.iter());
        let program = parser.parse_program()?;
        let mut g = Generator::new();
        g.register_native_fn(
            "print",
            builtin::builtin_print,
            vec![Type::Void],
            Type::Void,
        )?;
        let p = g.compile(&program)?;
        //println!("{:#?}", p);
        self.execute(&p)
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), Box<dyn Error>> {
        let main_fn = &program.functions[program.entry_function];
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
                let mut locals = vec![Value::Void; *local_var_cnt as usize];
                for code in codes {
                    //println!("{:?} {:?} {:?}", code, self.state.stack, locals);
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
                            let left = self.state.stack.pop().unwrap();
                            let right = self.state.stack.pop().unwrap();
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
                        ByteCode::Multiply => {
                            let left = self.state.stack.pop().unwrap();
                            let right = self.state.stack.pop().unwrap();
                            let val = match (left, right) {
                                (Value::Int(l), Value::Int(r)) => Value::Int(l * r),
                                (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                                _ => unreachable!(),
                            };
                            self.state.stack.push(val);
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
                    }
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
