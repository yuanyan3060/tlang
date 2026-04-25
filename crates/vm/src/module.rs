use std::collections::HashMap;
use crate::value::Value;

pub struct Module {
    pub exports: HashMap<String, Value>
}