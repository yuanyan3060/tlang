use std::error::Error;

use lex::Lex;
use parser::Parser;
use vm::semantic::Semantic;

fn main() -> Result<(), Box<dyn Error>> {
    let code = std::fs::read_to_string("main.td")?;
    let lex = Lex::new(code.chars());
    let tokens = lex.all();
    #[cfg(feature = "print_passes")]
    lex::pretty_print(&tokens);
    let mut parser = Parser::new(tokens.iter());
    let program = parser.parse_program()?;
    println!("{:#?}", program);
    let program = Semantic::new().analysis_type(&program).unwrap();
    println!("{:#?}", program);
    Ok(())
}
