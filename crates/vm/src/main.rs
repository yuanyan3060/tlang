use std::error::Error;

use lex::Lex;
use parser::Parser;
use vm::{Vm, compiler::{self, Compiler}, ir::IrBuilder, semantic::Semantic};

fn main() -> Result<(), Box<dyn Error>> {
    let code = std::fs::read_to_string("main.td")?;
    let lex = Lex::new(code.chars());
    let tokens = lex.all();
    #[cfg(feature = "print_passes")]
    lex::pretty_print(&tokens);
    let mut parser = Parser::new(tokens.iter());
    let program = parser.parse_program()?;
    println!("{:#?}", program);

    let pkg = compiler::compile(&program)?;
    let mut vm = Vm::new();
    vm.execute(&pkg)?;
    Ok(())
}
