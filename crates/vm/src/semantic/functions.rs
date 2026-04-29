use std::collections::HashMap;

use crate::semantic::SemanticError;
use crate::semantic::type_ast::FunctionDef;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FunctionId(u64);

pub struct FunctionTable {
    ids: HashMap<String, FunctionId>,
    function_defs: Vec<FunctionDef>,
}

impl FunctionTable {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            function_defs: Vec::new(),
        }
    }

    pub fn insert(&mut self, def: FunctionDef) -> Result<FunctionId, SemanticError> {
        if self.ids.contains_key(&def.name) {
            return Err(SemanticError::DuplicateDef {
                name: def.name.to_string(),
            });
        }

        let id = FunctionId(self.function_defs.len() as u64);
        self.ids.insert(def.name.to_string(), id);
        self.function_defs.push(def);

        Ok(id)
    }

    pub fn get(&self, id: FunctionId) -> Option<&FunctionDef> {
        self.function_defs.get(id.0 as usize)
    }

    pub fn get_mut(&mut self, id: FunctionId) -> Option<&mut FunctionDef> {
        self.function_defs.get_mut(id.0 as usize)
    }

    pub fn id(&self, name: &str) -> Option<FunctionId> {
        self.ids.get(name).copied()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&FunctionDef> {
        let id = self.ids.get(name)?;
        self.function_defs.get(id.0 as usize)
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut FunctionDef> {
        let id = self.ids.get(name)?;
        self.function_defs.get_mut(id.0 as usize)
    }
}
