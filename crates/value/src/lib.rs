use std::cell::RefCell;
use std::hash::{BuildHasher, DefaultHasher, Hash, Hasher};
use std::rc::Rc;

pub type NativeFnPtr = fn(&mut State, arg_cnt: u16) -> Value;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<String>),
    Object(Object),
    Fn(u32),
}

impl Value {
    pub fn as_obj(&mut self) -> Option<&mut Object> {
        if let Self::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_fn(&self) -> Option<u32> {
        if let Self::Fn(f) = self {
            Some(*f)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Type {
    Nil,
    Bool,
    Int,
    Float,
    String,
    // struct 索引
    Struct(u32),
    // function 索引
    Func(u32),
}

#[derive(Debug, Clone)]
pub struct Object {
    ty: Type,
    pub fields: Rc<RefCell<Vec<Value>>>,
}

impl Object {
    pub fn new(ty: Type, field_cnt: usize) -> Self {
        Self {
            ty,
            fields: Rc::new(RefCell::new(vec![Value::Nil; field_cnt])),
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => state.write_u8(0),
            Value::Bool(v) => {
                state.write_u8(1);
                v.hash(state);
            }
            Value::Int(v) => {
                state.write_u8(2);
                v.hash(state);
            }
            Value::Float(v) => {
                state.write_u8(3);
                state.write_u64(u64::from_ne_bytes(v.to_ne_bytes()));
            }
            Value::String(v) => {
                state.write_u8(4);
                v.hash(state);
            }
            Value::Object(v) => {
                state.write_u8(5);
                v.ty.hash(state);
                for field in &*v.fields.borrow() {
                    field.hash(state);
                }
            }
            Value::Fn(v) => {
                state.write_u8(6);
                v.hash(state);
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(l), Value::Bool(r)) => l.eq(r),
            (Value::Int(l), Value::Int(r)) => l.eq(r),
            (Value::Float(l), Value::Float(r)) => l.to_ne_bytes().eq(&r.to_ne_bytes()),
            (Value::String(l), Value::String(r)) => l.eq(r),
            (Value::Fn(l), Value::Fn(r)) => l.eq(r),
            (_, _) => false,
        }
    }
}

impl Eq for Value {}

impl BuildHasher for Value {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher::default()
    }
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }
}

pub struct State {
    pub locals: Vec<Value>,
    pub stack: Vec<Value>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(4096),
            locals: Vec::with_capacity(4096),
        }
    }
}
