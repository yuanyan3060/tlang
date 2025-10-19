use lex::Lex;

fn main() {
    let text = std::fs::read_to_string("main.td").unwrap();
    let lex = Lex::new(text.chars());
    let tokens = lex.all();
    lex::pretty_print(&tokens);
}
