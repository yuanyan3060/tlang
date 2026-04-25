use std::error::Error;

use ast::Program;

use crate::mem::{Mem, Tracer};
use crate::value::Value;

pub struct Vm {
    mem: Mem,
    tracer: Tracer,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            mem: Mem::new(),
            tracer: Tracer::new(),
        }
    }

    pub fn push_stack(&mut self, value: Value) {
        self.mem.push_stack(value);
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
