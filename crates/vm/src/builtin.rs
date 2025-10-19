use value::{State, Value};

pub fn builtin_print(state: &mut State) -> Value {
    let value = &state.stack.pop().unwrap();
    state.stack.pop();

    match value {
        Value::Void => println!("void"),
        Value::Nil => println!("nil"),
        Value::Bool(v) => println!("{}", v),
        Value::Int(v) => println!("{}", v),
        Value::Float(v) => println!("{}", v),
        Value::String(v) => println!("{}", v),
        Value::Object(_) => println!("Object"),
        Value::Fn(v) => println!("Fn({:X})", *v as usize),
        Value::Struct(v) => println!("Struct({:X})", *v as usize),
    }
    Value::Nil
}

pub fn builtin_timestamp(state: &mut State) -> Value {
    state.stack.pop();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Value::Float(now)
}
