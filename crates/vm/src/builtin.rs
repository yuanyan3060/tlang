use value::{State, Value};

pub fn builtin_print(state: &mut State, arg_cnt: u16) -> Value {
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

pub fn builtin_timestamp(state: &mut State, arg_cnt: u16) -> Value {
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
