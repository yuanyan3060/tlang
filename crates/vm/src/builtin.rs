use gc_arena::{Gc, Mutation};
use value::{State, Value};

pub fn builtin_print<'gc>(_: &'gc Mutation, state: &mut State, arg_cnt: u16) -> Value<'gc> {
    if state.stack.len() < arg_cnt as usize + 1 {
        panic!("builtin_print wrong arg")
    }

    let start = state.stack.len() - arg_cnt as usize;
    let iter = state.stack[start..].iter().enumerate();
    for (i, value) in iter {
        match value {
            Value::Nil => print!("nil"),
            Value::Bool(v) => print!("{}", v),
            Value::Int(v) => print!("{}", v),
            Value::Float(v) => print!("{}", v),
            Value::String(v) => print!("{}", v),
            Value::Object(_) => print!("Object"),
            Value::Fn(v) => print!("Fn({:X})", *v as usize),
        }

        if i + 1 != arg_cnt as usize {
            print!(" ")
        }
    }

    println!();
    for _ in 0..arg_cnt {
        state.stack.pop();
    }
    state.stack.pop();
    Value::Nil
}

pub fn builtin_str<'gc>(mc: &'gc Mutation<'gc>, state: &mut State, arg_cnt: u16) -> Value<'gc> {
    if arg_cnt != 1 {
        panic!("builtin_str wrong arg")
    }

    let val = match state.stack.pop() {
        Some(Value::Nil) => "nil".to_string(),
        Some(Value::Bool(v)) => v.to_string(),
        Some(Value::Int(v)) => v.to_string(),
        Some(Value::Float(v)) => v.to_string(),
        Some(Value::String(v)) => v.to_string(),
        Some(Value::Object(_)) => "Object".to_string(),
        Some(Value::Fn(v)) => format!("Fn({:X})", v),
        None => "nil".to_string(),
    };

    let val = Value::String(Gc::new(mc, val));
    state.stack.pop();
    val
}

pub fn builtin_timestamp<'gc>(_: &'gc Mutation, state: &mut State, arg_cnt: u16) -> Value<'gc> {
    for _ in 0..arg_cnt {
        state.stack.pop();
    }
    state.stack.pop();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Value::Float(now)
}

// 简单支持一下字符串格式化
pub fn builtin_str_format<'gc>(
    mc: &'gc Mutation<'gc>,
    state: &mut State,
    arg_cnt: u16,
) -> Value<'gc> {
    let mut args = Vec::new();
    for _ in 0..arg_cnt {
        let arg = state.stack.pop().unwrap_or(Value::Nil);
        args.push(arg);
    }

    if args.is_empty() {
        return Value::String(Gc::new(mc, "".to_string()));
    }

    let fstring = args.pop().unwrap();
    let fstring = fstring.as_str().unwrap();
    let mut output = "".to_string();
    let mut iter = fstring.chars().peekable();

    while let Some(c) = iter.next() {
        match c {
            '{' => {
                match iter.peek() {
                    Some('{') => {
                        iter.next();
                        output.push('{');
                    }
                    Some('}') => {
                        iter.next();
                        match args.pop() {
                            Some(v) => {
                                use std::fmt::Write;
                                _ = write!(&mut output, "{}", v);
                            }
                            None => output.push_str("{}"),
                        }
                    }
                    _ => {
                        output.push('{');
                    }
                }
                if iter.peek() == Some(&'{') {}
            }
            '}' => {
                output.push('}');
                iter.next_if(|c| *c == '}');
            }
            _ => output.push(c),
        }
    }

    state.stack.pop();

    Value::String(Gc::new(mc, output))
}
