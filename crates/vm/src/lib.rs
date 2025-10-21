use std::{error::Error, path::Path};

use bytecode::{
    ByteCode,
    generator::{Function, Generator, Program},
};
use gc_arena::{Arena, Gc, Rootable};
use lex::Lex;
use parser::Parser;

use value::{Object, State, Type, Value};

pub mod builtin;

pub struct Vm {
    arena: Arena<Rootable![State<'_>]>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            arena: Arena::<Rootable![State<'_>]>::new(|_| State::new()),
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

        g.register_native_fn("print", builtin::builtin_print, vec![Type::Nil], Type::Nil)?;
        g.register_native_fn("timestamp", builtin::builtin_timestamp, vec![], Type::Float)?;
        let p = g.compile(&program)?;
        #[cfg(debug_assertions)]
        println!("{:#?}", p);

        self.arena.mutate_root(|mc, state| {
            for c in &p.constants {
                state
                    .constants
                    .push(Value::String(Gc::new(mc, c.to_string())));
            }
        });
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
        Self::execute_codes(&mut self.arena, program, main_fn, 0)
    }

    pub fn execute_codes(
        arena: &mut Arena<Rootable![State<'_>]>,
        program: &Program,
        f: &Function,
        arg_cnt: u16,
    ) -> Result<(), Box<dyn Error>> {
        arena.collect_debt();
        match f {
            Function::Native { func, .. } => arena.mutate_root(|mc, state| {
                let val = func(mc, state, arg_cnt);
                state.stack.push(val);
            }),
            Function::Custom {
                codes,
                local_var_cnt,
                ..
            } => {
                let local_offset = arena.mutate_root(|_, state| {
                    let local_offset = state.locals.len();
                    for _ in 0..*local_var_cnt {
                        state.locals.push(Value::Nil);
                    }
                    local_offset
                });

                let mut code_offset = 0;
                while code_offset < codes.len() {
                    let call = arena.mutate_root(|mc, state| {
                        while code_offset < codes.len() {
                            let code = &codes[code_offset];
                            #[cfg(debug_assertions)]
                            {
                                let locals = &mut state.locals[local_offset..];
                                println!(
                                    "{:?} {:?} {:?} {}",
                                    code,
                                    state.stack,
                                    locals,
                                    mc.metrics().total_gc_allocation(),
                                );
                            }

                            match code {
                                ByteCode::Pop => {
                                    state.stack.pop();
                                }
                                ByteCode::LoadNil => {
                                    state.stack.push(Value::Nil);
                                }
                                ByteCode::LoadBool { val } => {
                                    state.stack.push(Value::Bool(*val));
                                }
                                ByteCode::LoadInt { val } => {
                                    state.stack.push(Value::Int(*val));
                                }
                                ByteCode::LoadFloat { val } => {
                                    state.stack.push(Value::Float(*val));
                                }
                                ByteCode::LoadString { idx } => {
                                    let val = state.constants[*idx as usize];
                                    state.stack.push(val);
                                }
                                ByteCode::LoadFunction { idx } => {
                                    state.stack.push(Value::Fn(*idx));
                                }
                                ByteCode::NewStruct { idx, cnt } => {
                                    state.stack.push(Value::Object(Gc::new(
                                        mc,
                                        Object::new(mc, Type::Struct(*idx), *cnt as usize),
                                    )));
                                }
                                ByteCode::Store { idx } => {
                                    let val = state.stack.pop().unwrap();
                                    let locals = &mut state.locals[local_offset..];
                                    locals[*idx as usize] = val;
                                }
                                ByteCode::Load { idx } => {
                                    let locals = &mut state.locals[local_offset..];
                                    state.stack.push(locals[*idx as usize]);
                                }
                                ByteCode::GetField { offset } => {
                                    let obj = state.stack.pop().unwrap();

                                    let field =
                                        obj.as_obj().unwrap().fields.borrow()[*offset as usize];
                                    state.stack.push(field);
                                }
                                ByteCode::SetField { offset } => {
                                    let val = state.stack.pop().unwrap();
                                    let target = state.stack.pop().unwrap();
                                    target.as_obj().unwrap().fields.borrow_mut(mc)
                                        [*offset as usize] = val;
                                }
                                ByteCode::Add => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l + r),
                                        (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
                                        (Value::String(l), Value::String(r)) => {
                                            Value::String(Gc::new(mc, format!("{}{}", r, l)))
                                        }
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Subtract => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l - r),
                                        (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
                                        (Value::String(l), Value::String(r)) => {
                                            Value::String(Gc::new(mc, format!("{}{}", r, l)))
                                        }
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Multiply => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l * r),
                                        (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Divide => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l / r),
                                        (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Modulo => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l % r),
                                        (Value::Float(l), Value::Float(r)) => Value::Float(l % r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::BitAnd => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l & r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::BitOr => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l | r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::BitXor => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Int(l ^ r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Equal => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Bool(l == r),
                                        (Value::Float(l), Value::Float(r)) => Value::Bool(l == r),
                                        (Value::String(l), Value::String(r)) => Value::Bool(l == r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::NotEqual => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Bool(l != r),
                                        (Value::Float(l), Value::Float(r)) => Value::Bool(l != r),
                                        (Value::String(l), Value::String(r)) => Value::Bool(l != r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Less => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Bool(l < r),
                                        (Value::Float(l), Value::Float(r)) => Value::Bool(l < r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Greater => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Bool(l > r),
                                        (Value::Float(l), Value::Float(r)) => Value::Bool(l > r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::LessEqual => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Bool(l <= r),
                                        (Value::Float(l), Value::Float(r)) => Value::Bool(l <= r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::GreaterEqual => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Int(l), Value::Int(r)) => Value::Bool(l >= r),
                                        (Value::Float(l), Value::Float(r)) => Value::Bool(l >= r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::And => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Bool(l), Value::Bool(r)) => Value::Bool(l && r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Or => {
                                    let right = state.stack.pop().unwrap();
                                    let left = state.stack.pop().unwrap();
                                    let val = match (left, right) {
                                        (Value::Bool(l), Value::Bool(r)) => Value::Bool(l || r),
                                        _ => unreachable!(),
                                    };
                                    state.stack.push(val);
                                }
                                ByteCode::Minus => {
                                    let val = state.stack.pop().unwrap();
                                    match val {
                                        Value::Int(v) => {
                                            state.stack.push(Value::Int(-v));
                                        }
                                        Value::Float(v) => {
                                            state.stack.push(Value::Float(-v));
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                ByteCode::Not => {
                                    let val = state.stack.pop().unwrap();
                                    match val {
                                        Value::Bool(v) => {
                                            state.stack.push(Value::Bool(!v));
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                ByteCode::BitNot => {
                                    let val = state.stack.pop().unwrap();
                                    match val {
                                        Value::Int(v) => {
                                            state.stack.push(Value::Int(!v));
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                ByteCode::GetIndex => todo!(),
                                ByteCode::Call { arg_cnt } => {
                                    let idx = state.stack.len() - *arg_cnt as usize - 1;
                                    let f = state.stack[idx].as_fn();

                                    if let Some(f) = f {
                                        code_offset += 1;
                                        return Some(Call {
                                            fn_idx: f,
                                            arg_cnt: *arg_cnt,
                                        });
                                        
                                    }
                                }
                                ByteCode::Swap => {
                                    let i = state.stack.len() - 1;
                                    let j = i - 1;
                                    state.stack.swap(i, j);
                                }
                                ByteCode::Return => {
                                    code_offset = usize::MAX - 1;
                                    break;
                                }
                                ByteCode::JumpIfFalse { offset } => {
                                    let val = state.stack.pop().unwrap().as_bool().unwrap();
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
                        None
                    });

                    if let Some(call) = call {
                        let f = &program.functions[call.fn_idx as usize];
                        Self::execute_codes(arena, program, f, call.arg_cnt)?;
                    }
                }

                arena.mutate_root(|_, state| {
                    for _ in 0..*local_var_cnt {
                        state.locals.pop();
                    }
                });
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

pub struct Call {
    fn_idx: u32,
    arg_cnt: u16,
}
