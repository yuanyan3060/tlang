use value::{State, Value};

pub fn builtin_print(state: &mut State) -> Value {
    let value = &state.stack.pop().unwrap();
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
