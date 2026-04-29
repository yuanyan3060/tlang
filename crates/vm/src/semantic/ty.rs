use indexmap::IndexMap;

use crate::semantic::SemanticError;
use crate::semantic::structs::StructId;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TypeId(u64);

impl TypeId {
    pub const NIL: Self = TypeId(0);
    pub const BOOL: Self = TypeId(1);
    pub const INT: Self = TypeId(2);
    pub const FLOAT: Self = TypeId(3);
    pub const STRING: Self = TypeId(4);
}

pub struct TypeTable {
    kinds: IndexMap<TypeKind, TypeId>,
}

impl TypeTable {
    pub fn new() -> Self {
        let mut pool = Self {
            kinds: IndexMap::new(),
        };

        pool.intern(TypeKind::Nil);
        pool.intern(TypeKind::Bool);
        pool.intern(TypeKind::Int);
        pool.intern(TypeKind::Float);
        pool.intern(TypeKind::String);

        pool
    }

    pub fn get(&self, id: TypeId) -> Option<&TypeKind> {
        self.kinds.get_index(id.0 as usize).map(|(k, _)| k)
    }

    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.kinds.get(&kind) {
            return id;
        }

        let id = TypeId(self.kinds.len() as u64);
        self.kinds.insert(kind, id);
        id
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TypeKind {
    Nil,
    Bool,
    Int,
    Float,
    String,
    Struct(StructId),
    Vec {
        element: TypeId,
    },
    NativeFunction {
        args: Vec<TypeId>,
        return_ty: TypeId,
    },
    Fn {
        args: Vec<TypeId>,
        return_ty: TypeId,
    },
}

impl TypeKind {
    pub fn as_callable(&self) -> Option<(&[TypeId], TypeId)> {
        match self {
            TypeKind::NativeFunction { args, return_ty } => Some((args, *return_ty)),
            TypeKind::Fn { args, return_ty } => Some((args, *return_ty)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GenericSlot(pub u64);

#[derive(Debug, Clone, Copy)]
pub enum TypeSlot {
    Static(TypeId),
    Dyn(GenericSlot),
}

#[derive(Debug, Clone)]
pub struct GenericFn {
    pub args: Vec<TypeSlot>,
    pub return_ty: Option<TypeSlot>,
}

impl GenericFn {
    pub fn monomorphization(&self, input_args: &[TypeId]) -> Result<TypeKind, SemanticError> {
        assert_eq!(input_args.len(), self.args.len());

        let mut args = Vec::new();
        let mut bounds = vec![None; self.args.len()];

        for idx in 0..self.args.len() {
            let arg = &self.args[idx];
            let input = input_args[idx];

            match arg {
                TypeSlot::Static(type_id) => {
                    assert_eq!(input, *type_id);
                    *type_id
                }
                TypeSlot::Dyn(generic_type) => {
                    match bounds[generic_type.0 as usize] {
                        Some(bound) => assert_eq!(bound, input),
                        None => bounds[generic_type.0 as usize] = Some(input),
                    }
                    input
                }
            };

            args.push(input);
        }

        let return_ty = match self.return_ty {
            Some(TypeSlot::Static(type_id)) => type_id,
            Some(TypeSlot::Dyn(generic_type)) => bounds[generic_type.0 as usize].unwrap(),
            None => TypeId::NIL,
        };

        Ok(TypeKind::NativeFunction { args, return_ty })
    }
}

#[derive(Debug)]
pub enum GenericType {
    Vec,
}

impl GenericType {
    pub fn monomorphization(&self, param: &[TypeId]) -> Result<TypeKind, SemanticError> {
        match self {
            GenericType::Vec => {
                assert_eq!(param.len(), 1);
                Ok(TypeKind::Vec { element: param[0] })
            }
        }
    }
}
