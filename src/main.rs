mod tokens;
mod lexer;
mod parser;
mod ast;
mod errors;
mod utils;

// Tests
mod tests;

use lexer::Lexer;
use parser::Parser;

fn main() -> Result<(), ()> {
    let source = "let x = 1.19; y = 0.5; let z;\n( ( ( x + y ) ** z ^ x ) * y - ( x / y | z ) );";
    let lexer = Lexer::new(source)?;
    let mut parser = Parser::new(lexer);
    parser.generate_expressions();
    // println!("{:?}", lexer.next());
    // println!("{:?}", lexer.next());
    // println!("{:?}", lexer.next());
    // println!("{:?}", lexer.next());
    Ok(())
}
