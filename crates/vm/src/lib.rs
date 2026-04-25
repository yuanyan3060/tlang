mod builtin;
mod bytecode;
pub mod compiler;
mod mem;
mod module;
mod package;
mod value;
mod vm;
pub mod ir;

pub mod semantic;

pub use vm::Vm;

/*use std::{error::Error, path::Path, time::Duration};

use bytecode::{
    ByteCode,
    generator::{Function, Generator, Program},
};
use gc_arena::{Arena, Gc, Rootable};
use lex::Lex;
use parser::Parser;

use value::{Context, Object, State, Type, Value};

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
        #[cfg(feature = "print_passes")]
        lex::pretty_print(&tokens);
        let mut parser = Parser::new(tokens.iter());
        let program = parser.parse_program()?;
        let mut g = Generator::new();

        g.register_native_fn(
            "print",
            Box::new(builtin::builtin_print),
            vec![Type::Nil],
            Type::Nil,
        )?;
        g.register_native_fn(
            "timestamp",
            Box::new(builtin::builtin_timestamp),
            vec![],
            Type::Float,
        )?;
        g.register_native_fn(
            "str",
            Box::new(builtin::builtin_str),
            vec![Type::Nil],
            Type::String,
        )?;
        g.register_native_fn(
            "str::format",
            Box::new(builtin::builtin_str_format),
            vec![Type::Nil],
            Type::String,
        )?;

        let p = g.compile(&program)?;
        #[cfg(feature = "print_passes")]
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
        #[cfg(feature = "print_passes")]
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
        match f {
            Function::Native { func, .. } => arena.mutate_root(|mc, state| {
                let ctx = Context { mc, state };
                func(ctx, arg_cnt);
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
                    let code = &codes[code_offset];
                    #[cfg(feature = "print_passes")]
                    {
                        arena.mutate_root(|mc, state| {
                            let locals = &mut state.locals[local_offset..];
                            println!(
                                "{:?} {:?} {:?} {}",
                                code,
                                state.stack,
                                locals,
                                mc.metrics().total_gc_allocation()
                            );
                        });
                    }

                    match code {
                        ByteCode::Pop => {
                            arena.mutate_root(|_, state| {
                                state.stack.pop();
                            });
                        }
                        ByteCode::LoadNil => {
                            arena.mutate_root(|_, state| {
                                state.stack.push(Value::Nil);
                            });
                        }
                        ByteCode::LoadBool { val } => {
                            arena.mutate_root(|_, state| {
                                state.stack.push(Value::Bool(*val));
                            });
                        }
                        ByteCode::LoadInt { val } => {
                            arena.mutate_root(|_, state| {
                                state.stack.push(Value::Int(*val));
                            });
                        }
                        ByteCode::LoadFloat { val } => {
                            arena.mutate_root(|_, state| {
                                state.stack.push(Value::Float(*val));
                            });
                        }
                        ByteCode::LoadString { idx } => {
                            arena.mutate_root(|_, state| {
                                let val = state.constants[*idx as usize];
                                state.stack.push(val);
                            });
                        }
                        ByteCode::LoadFunction { idx } => {
                            arena.mutate_root(|_, state| {
                                state.stack.push(Value::Fn(*idx));
                            });
                        }
                        ByteCode::NewStruct { idx, cnt } => {
                            arena.mutate_root(|mc, state| {
                                state.stack.push(Value::Struct(Gc::new(
                                    mc,
                                    Object::new(mc, *idx, *cnt as usize),
                                )));
                            });
                        }
                        ByteCode::Store { idx } => {
                            arena.mutate_root(|_, state| {
                                let val = state.stack.pop().unwrap();
                                let locals = &mut state.locals[local_offset..];
                                locals[*idx as usize] = val;
                            });
                        }
                        ByteCode::Load { idx } => {
                            arena.mutate_root(|_, state| {
                                let locals = &mut state.locals[local_offset..];
                                state.stack.push(locals[*idx as usize]);
                            });
                        }
                        ByteCode::GetField { offset } => {
                            arena.mutate_root(|_, state| {
                                let obj = state.stack.pop().unwrap();

                                let field = obj.as_obj().unwrap().fields.borrow()[*offset as usize];
                                state.stack.push(field);
                            });
                        }
                        ByteCode::SetField { offset } => {
                            arena.mutate_root(|mc, state| {
                                let val = state.stack.pop().unwrap();
                                let target = state.stack.last_mut().unwrap();
                                target.as_obj().unwrap().fields.borrow_mut(mc)[*offset as usize] =
                                    val;
                            });
                        }
                        ByteCode::Add => {
                            arena.mutate_root(|mc, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l + r),
                                    (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
                                    (Value::String(l), Value::String(r)) => {
                                        Value::String(Gc::new(mc, format!("{}{}", l, r)))
                                    }
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Subtract => {
                            arena.mutate_root(|mc, state| {
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
                            });
                        }
                        ByteCode::Multiply => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l * r),
                                    (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Divide => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l / r),
                                    (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Modulo => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l % r),
                                    (Value::Float(l), Value::Float(r)) => Value::Float(l % r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::BitAnd => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l & r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::BitOr => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l | r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::BitXor => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Int(l ^ r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Equal => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Bool(l == r),
                                    (Value::Float(l), Value::Float(r)) => Value::Bool(l == r),
                                    (Value::String(l), Value::String(r)) => Value::Bool(l == r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::NotEqual => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Bool(l != r),
                                    (Value::Float(l), Value::Float(r)) => Value::Bool(l != r),
                                    (Value::String(l), Value::String(r)) => Value::Bool(l != r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Less => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Bool(l < r),
                                    (Value::Float(l), Value::Float(r)) => Value::Bool(l < r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Greater => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Bool(l > r),
                                    (Value::Float(l), Value::Float(r)) => Value::Bool(l > r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::LessEqual => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Bool(l <= r),
                                    (Value::Float(l), Value::Float(r)) => Value::Bool(l <= r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::GreaterEqual => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Int(l), Value::Int(r)) => Value::Bool(l >= r),
                                    (Value::Float(l), Value::Float(r)) => Value::Bool(l >= r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::And => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Bool(l), Value::Bool(r)) => Value::Bool(l && r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Or => {
                            arena.mutate_root(|_, state| {
                                let right = state.stack.pop().unwrap();
                                let left = state.stack.pop().unwrap();
                                let val = match (left, right) {
                                    (Value::Bool(l), Value::Bool(r)) => Value::Bool(l || r),
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::Minus => {
                            arena.mutate_root(|_, state| {
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
                            });
                        }
                        ByteCode::Not => {
                            arena.mutate_root(|_, state| {
                                let val = state.stack.pop().unwrap();
                                match val {
                                    Value::Bool(v) => {
                                        state.stack.push(Value::Bool(!v));
                                    }
                                    _ => unreachable!(),
                                }
                            });
                        }
                        ByteCode::BitNot => {
                            arena.mutate_root(|_, state| {
                                let val = state.stack.pop().unwrap();
                                match val {
                                    Value::Int(v) => {
                                        state.stack.push(Value::Int(!v));
                                    }
                                    _ => unreachable!(),
                                }
                            });
                        }
                        ByteCode::GetIndex => {
                            arena.mutate_root(|_, state| {
                                let idx = state.stack.pop().unwrap();
                                let vec = state.stack.pop().unwrap();
                                let val = match (vec, idx) {
                                    (Value::Vec(vec), Value::Int(idx)) => {
                                        vec.fields.borrow()[idx as usize]
                                    }
                                    _ => unreachable!(),
                                };
                                state.stack.push(val);
                            });
                        }
                        ByteCode::SetIndex => {
                            arena.mutate_root(|mc, state| {
                                let val = state.stack.pop().unwrap();
                                let idx = state.stack.pop().unwrap();
                                let vec = state.stack.pop().unwrap();
                                match (vec, idx) {
                                    (Value::Vec(vec), Value::Int(idx)) => {
                                        vec.fields.borrow_mut(mc)[idx as usize] = val;
                                    }
                                    _ => unreachable!(),
                                };
                            });
                        }
                        ByteCode::Call { arg_cnt } => {
                            let mut need_gc = false;
                            let f = arena.mutate_root(|_, state| {
                                need_gc = check_need_gc(state);
                                let idx = state.stack.len() - *arg_cnt as usize - 1;
                                state.stack[idx].as_fn()
                            });

                            if need_gc {
                                arena.collect_debt();
                            }

                            if let Some(f) = f {
                                let f = &program.functions[f as usize];
                                Self::execute_codes(arena, program, f, *arg_cnt)?;
                            }
                        }
                        ByteCode::Swap => {
                            arena.mutate_root(|_, state| {
                                let i = state.stack.len() - 1;
                                let j = i - 1;
                                state.stack.swap(i, j);
                            });
                        }
                        ByteCode::Return => {
                            break;
                        }
                        ByteCode::JumpIfFalse { offset } => {
                            let mut need_gc = false;
                            let val = arena.mutate_root(|_, state| {
                                need_gc = check_need_gc(state);
                                state.stack.pop().unwrap().as_bool().unwrap()
                            });
                            if need_gc {
                                arena.collect_debt();
                            }
                            if !val {
                                code_offset = *offset as usize;
                                continue;
                            }
                        }
                        ByteCode::Jump { offset } => {
                            let mut need_gc = false;
                            arena.mutate_root(|_, state| {
                                need_gc = check_need_gc(state);
                            });
                            if need_gc {
                                arena.collect_debt();
                            }
                            code_offset = *offset as usize;

                            continue;
                        }
                        ByteCode::Nop => {}
                    }

                    code_offset += 1;
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

fn check_need_gc(state: &mut State) -> bool {
    state.jump_cnt += 1;
    if state.jump_cnt < 1000000 {
        return false;
    }

    state.jump_cnt = 0;
    let now = std::time::Instant::now();
    if now - state.last_collect_time.0 > Duration::from_secs(120) {
        return true;
    }
    false
}
*/
