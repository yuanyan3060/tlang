use std::collections::HashMap;

use crate::semantic::SemanticError;
use crate::semantic::ty::{GenericFn, TypeId};

#[derive(Debug)]
pub enum Indent {
    Local { name: String, ty: TypeId, idx: usize },
    GenericFn { name: String, f: GenericFn, idx: usize },
}

impl Indent {
    pub fn ty(&self) -> Option<TypeId> {
        match self {
            Indent::Local { ty, .. } => Some(*ty),
            _ => None,
        }
    }
}

pub enum Location {
    Local(usize),
    Global(usize)
}

pub enum SymbolKind {
    Normal {
        type_id: TypeId,
    },
    GenericFn {
        func: GenericFn
    }
}

pub struct Symbol {
    pub name: String,
    pub location: Location,
    pub kind: SymbolKind
}

impl Symbol {
    pub fn ty(&self) -> Option<TypeId> {
        match &self.kind {
            SymbolKind::Normal { type_id } => Some(*type_id),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    parent: Option<usize>,
    idents: HashMap<String, Indent>,
}

pub struct SymbolTable{
    global: HashMap<String, Symbol>,
    locals: Vec<HashMap<String, Symbol>>,
    next_local: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            global: HashMap::new(),
            locals: Vec::new(),
            next_local: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if let Some(local) = self.locals.pop() {
            self.next_local -= local.len()
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let scopes = self.locals.iter().chain([&self.global]).rev();

        for scope in scopes {
            if let Some(ident) = scope.get(name) {
                return Some(ident);
            }
        }
        None
    }

    pub fn insert(&mut self, name: &str, type_id: TypeId) -> Result<(), SemanticError> {
        let (scope, location) = match self.locals.last_mut() {
            Some(scope) => {
                let loc = self.next_local;
                self.next_local += 1;
                (scope, Location::Local(loc))
            },
            None => {
                let loc = Location::Global(self.global.len());
                (&mut self.global, loc)
            },
        };

        if scope.contains_key(name) {
            todo!()
        }

        let symbol = Symbol {
            name: name.to_string(),
            location,
            kind: SymbolKind::Normal { type_id },
        };

        scope.insert(
            name.to_string(),
            symbol
        );

        Ok(())
    }

    pub fn insert_generic_fn(&mut self, name: &str, func: GenericFn) -> Result<(), SemanticError> {
        let (scope, location) = match self.locals.last_mut() {
            Some(scope) => {
                let loc = self.next_local;
                self.next_local += 1;
                (scope, Location::Local(loc))
            },
            None => {
                let loc = Location::Global(self.global.len());
                (&mut self.global, loc)
            },
        };

        if scope.contains_key(name) {
            todo!()
        }

        let symbol = Symbol {
            name: name.to_string(),
            location,
            kind: SymbolKind::GenericFn { func },
        };

        scope.insert(
            name.to_string(),
            symbol
        );

        Ok(())
    }
}
