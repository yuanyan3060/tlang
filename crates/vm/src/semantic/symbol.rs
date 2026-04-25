use std::collections::HashMap;

use crate::semantic::SemanticError;
use crate::semantic::ty::{StructId};
use crate::value::Value;

pub struct SymbolTable {
    structs: HashMap<String, StructId>,
    values: HashMap<String, Value>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn lookup_ty(&self, name: &str) -> Result<Type, SemanticError> {
        self.structs
            .get(name)
            .copied()
            .ok_or_else(|| SemanticError::UnknownType {
                name: name.to_string(),
            })
    }

    pub fn lookup_value(&self, name: &str) -> Result<Value, SemanticError> {
        self.values
            .get(name)
            .copied()
            .ok_or_else(|| SemanticError::UnknownType {
                name: name.to_string(),
            })
    }

    pub fn contains_ty(&mut self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    pub fn contains_value(&mut self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn insert_ty(&mut self, name: &str, ty: Type) -> Option<Type> {
        self.structs.insert(name.to_string(), ty)
    }

    pub fn insert_value(&mut self, name: &str, value: Value) -> Option<Value> {
        self.values.insert(name.to_string(), value)
    }
}
