use std::fmt::Display;
use std::hash::{BuildHasher, DefaultHasher, Hash, Hasher};

use gc_arena::lock::{GcRefLock, RefLock};
use gc_arena::{Collect, Gc, Mutation};

pub type NativeFnPtr = for<'a> fn(&'a Mutation<'a>, &mut State, arg_cnt: u16) -> Value<'a>;

#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
pub enum Value<'gc> {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Gc<'gc, String>),
    Object(Gc<'gc, Object<'gc>>),
    Fn(u32),
}

impl<'gc> Value<'gc> {
    pub fn as_obj(&self) -> Option<Gc<'gc, Object<'gc>>> {
        if let Self::Object(obj) = self {
            Some(*obj)
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

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Collect)]
#[collect(no_drop)]
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

#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Object<'gc> {
    ty: u32,
    pub fields: GcRefLock<'gc, Vec<Value<'gc>>>,
}

impl<'gc> Object<'gc> {
    pub fn new(mc: &'gc Mutation<'gc>, ty: u32, field_cnt: usize) -> Self {
        Self {
            ty,
            fields: GcRefLock::new(mc, RefLock::new(vec![Value::Nil; field_cnt])),
        }
    }
}

impl<'gc> Hash for Value<'gc> {
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

impl<'gc> PartialEq for Value<'gc> {
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

impl<'gc> Eq for Value<'gc> {}

impl<'gc> BuildHasher for Value<'gc> {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher::default()
    }
}

impl<'gc> Display for Value<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Object(v) => write!(f, "Object({})", v.ty),
            Value::Fn(v) => write!(f, "Fn({})", v),
        }
    }
}

pub struct Instant(pub std::time::Instant);

unsafe impl Collect for Instant {
    fn needs_trace() -> bool
    where
        Self: Sized,
    {
        true
    }

    fn trace(&self, _cc: &gc_arena::Collection) {}
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct State<'gc> {
    pub locals: Vec<Value<'gc>>,
    pub stack: Vec<Value<'gc>>,
    pub constants: Vec<Value<'gc>>,
    pub jump_cnt: u32,
    pub last_collect_time: Instant,
}

impl<'gc> Default for State<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> State<'gc> {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(4096),
            locals: Vec::with_capacity(4096),
            constants: Vec::new(),
            jump_cnt: 0,
            last_collect_time: Instant(std::time::Instant::now()),
        }
    }
}
