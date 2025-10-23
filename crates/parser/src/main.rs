use std::error::Error;

use lex::Lex;
use parser::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let code = std::fs::read_to_string("struct_define.td")?;
    let lex = Lex::new(code.chars());
    let tokens = lex.all();
    #[cfg(debug_assertions)]
    lex::pretty_print(&tokens);
    let mut parser = Parser::new(tokens.iter());
    let program = parser.parse_program()?;
    println!("{:#?}", program);
    Ok(())
}
