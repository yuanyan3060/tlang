use crate::vm::Vm;

pub type NativeFn = fn(&mut Vm);

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(GcHandle),
    Struct(GcHandle),
    Vec(GcHandle),
    NativeFn(NativeFn),
    Fn(u32),
}

impl Value {
    pub fn as_handle(&self) -> Option<GcHandle> {
        match self {
            Value::String(gc_handle) => Some(*gc_handle),
            Value::Struct(gc_handle) => Some(*gc_handle),
            Value::Vec(gc_handle) => Some(*gc_handle),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct GcHandle(slotmap::KeyData);

impl From<slotmap::KeyData> for GcHandle {
    fn from(value: slotmap::KeyData) -> Self {
        Self(value)
    }
}

unsafe impl slotmap::Key for GcHandle {
    fn data(&self) -> slotmap::KeyData {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum GcMark {
    White,
    Gray,
    Black,
}

#[derive(Debug)]
pub enum GcData {
    String(String),
    Vec(Vec<Value>),
    Struct(Vec<Value>),
}

#[derive(Debug)]
pub struct GcValue {
    pub mark: GcMark,
    pub data: GcData,
}
