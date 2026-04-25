use ast::StructDef;

use crate::bytecode::ByteCode;
use crate::value::NativeFn;

pub struct Package {
    pub constants: Vec<String>,
    pub structs: Vec<StructDef>,
    pub functions: Vec<Function>,
    pub entry_function: usize,
}

pub enum Function {
    Native {
        name: String,
        func: NativeFn,
    },
    Custom {
        name: String,
        codes: Vec<ByteCode>,
        local_var_cnt: u32,
    },
}
