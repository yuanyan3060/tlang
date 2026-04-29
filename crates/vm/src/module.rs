use crate::value::Value;
use std::collections::HashMap;

pub struct Module {
    pub exports: HashMap<String, Value>,
}
