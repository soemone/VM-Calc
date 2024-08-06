mod tokens;
mod lexer;

// Tests
mod tests;

use lexer::Lexer;

fn main() -> Result<(), ()> {
    let source = "* *= ** **=";
    let mut lexer = Lexer::new(source)?;
    println!("{:?}", lexer.next());
    println!("{:?}", lexer.next());
    println!("{:?}", lexer.next());
    Ok(())
}
