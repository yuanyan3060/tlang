use ast::StructDef;

use crate::bytecode::ByteCode;
use crate::compiler::ConstValue;
use crate::semantic::structs::StructTable;
use crate::value::{NativeFn, Value};

pub struct Package {
    pub constants: Vec<ConstValue>,
    pub global: Vec<Value>,
    pub structs: StructTable,
    pub functions: Vec<Function>,
    pub entry_function: usize,
}

#[derive(Debug)]
pub enum Function {
    Native {
        name: String,
        func: NativeFn,
    },
    Custom {
        name: String,
        codes: Vec<ByteCode>,
        local_var_cnt: u32,
        temp_var_cnt: u32,
    },
}
