use std::collections::HashMap;

use crate::semantic::SemanticError;
use crate::semantic::type_ast::StructDef;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StructId(u64);

pub struct StructTable {
    ids: HashMap<String, StructId>,
    struct_defs: Vec<StructDef>,
}

impl StructTable {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            struct_defs: Vec::new(),
        }
    }

    pub fn insert(&mut self, def: StructDef) -> Result<StructId, SemanticError> {
        if self.ids.contains_key(&def.name) {
            return Err(SemanticError::DuplicateDef {
                name: def.name.to_string(),
            });
        }

        let id = StructId(self.struct_defs.len() as u64);
        self.ids.insert(def.name.to_string(), id);
        self.struct_defs.push(def);

        Ok(id)
    }

    pub fn get(&self, id: StructId) -> Option<&StructDef> {
        self.struct_defs.get(id.0 as usize)
    }

    pub fn get_mut(&mut self, id: StructId) -> Option<&mut StructDef> {
        self.struct_defs.get_mut(id.0 as usize)
    }

    pub fn id(&self, name: &str) -> Option<StructId> {
        self.ids.get(name).copied()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&StructDef> {
        let id = self.ids.get(name)?;
        self.struct_defs.get(id.0 as usize)
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut StructDef> {
        let id = self.ids.get(name)?;
        self.struct_defs.get_mut(id.0 as usize)
    }
}
