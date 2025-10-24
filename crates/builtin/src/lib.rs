use gc_arena::Gc;
use value::{Context, NativeFn, Object, Value};

pub fn builtin_print(ctx: Context, arg_cnt: u16) {
    if ctx.state.stack.len() < arg_cnt as usize + 1 {
        panic!("builtin_print wrong arg")
    }

    let start = ctx.state.stack.len() - arg_cnt as usize;
    let iter = ctx.state.stack[start..].iter().enumerate();
    for (i, value) in iter {
        print!("{}", value);
        if i + 1 != arg_cnt as usize {
            print!(" ")
        }
    }

    println!();
    for _ in 0..arg_cnt {
        ctx.state.stack.pop();
    }
    ctx.state.stack.pop();
    ctx.state.stack.push(Value::Nil);
}

pub fn builtin_str(ctx: Context, arg_cnt: u16) {
    if arg_cnt != 1 {
        panic!("builtin_str wrong arg")
    }

    let val = match ctx.state.stack.pop() {
        Some(v) => v.to_string(),
        None => "nil".to_string(),
    };

    let val = Value::String(Gc::new(ctx.mc, val));
    ctx.state.stack.pop();
    ctx.state.stack.push(val);
}

pub fn builtin_timestamp(ctx: Context, arg_cnt: u16) {
    for _ in 0..arg_cnt {
        ctx.state.stack.pop();
    }
    ctx.state.stack.pop();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    ctx.state.stack.push(Value::Float(now));
}

// 简单支持一下字符串格式化
pub fn builtin_str_format(ctx: Context, arg_cnt: u16) {
    let mut args = Vec::new();
    for _ in 0..arg_cnt {
        let arg = ctx.state.stack.pop().unwrap_or(Value::Nil);
        args.push(arg);
    }

    if args.is_empty() {
        ctx.state.stack.pop();
        ctx.state
            .stack
            .push(Value::String(Gc::new(ctx.mc, "".to_string())));
        return;
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

    ctx.state.stack.pop();

    ctx.state.stack.push(Value::String(Gc::new(ctx.mc, output)));
}

pub fn builtin_vec_new(ty: u32) -> NativeFn {
    Box::new(move |ctx, _| {
        ctx.state.stack.pop();
        ctx.state
            .stack
            .push(Value::Vec(Gc::new(ctx.mc, Object::new(ctx.mc, ty, 0))));
    })
}

pub fn builtin_vec_len() -> NativeFn {
    Box::new(move |ctx, _| {
        let vec = ctx.state.stack.pop();
        ctx.state.stack.pop();

        let size = match vec {
            Some(Value::Vec(x)) => x.fields.borrow().len(),
            _ => 0,
        };
        ctx.state.stack.push(Value::Int(size as i64));
    })
}

pub fn builtin_vec_push(ctx: Context, _: u16) {
    let e = ctx.state.stack.pop().unwrap_or(Value::Nil);
    let vec = ctx.state.stack.pop();

    if let Some(Value::Vec(x)) = vec {
        x.fields.borrow_mut(ctx.mc).push(e);
    }
    ctx.state.stack.pop();
    ctx.state.stack.push(Value::Nil);
}
