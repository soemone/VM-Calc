#[cfg(test)]

// These are just tests. Nothing to see here, that is if physical laws are still the same. I hope they are, at least.
// If not, this place is gonna need renovation.
mod tests {
    use std::{ops::Range, rc::Rc};
    use crate::{ast::{Tree, AST}, errors::Error, lexer::Lexer, parser::Parser, utils::Span};
    use super::*;

    fn generate_tree(input: &str) -> Vec<Result<Rc<Tree<'_>>, Error>> {
        let lexer = Lexer::new(input).unwrap();
        let mut parser = Parser::new(lexer);
        parser.generate_expressions()
    }

    fn ok_tree(ast: AST, range: Range<usize>) -> Result<Rc<Tree>, Error> {
        Ok(Rc::new(Tree::new(ast, Span::from_range(range))))
    }

    fn expect_error<T, A>(value: &Result<T, A>) {
        assert!(matches!(value, Err(..)));
    }

    #[test] 
    fn number() {
        let tree = generate_tree("6;0.5;.9;");
        assert_eq!(tree[0], ok_tree(AST::Number { value: 6.0 }, 0..1));
        assert_eq!(tree[1], ok_tree(AST::Number { value: 0.5 }, 2..5));
        assert_eq!(tree[2], ok_tree(AST::Number { value: 0.9 }, 6..8));
    }

    #[test] 
    fn hex() {
        let tree = generate_tree("0x0;0xf;_ 0x;");
        assert_eq!(tree[0], ok_tree(AST::Number { value: 0.0 }, 0..3));
        assert_eq!(tree[1], ok_tree(AST::Number { value: 15.0 }, 4..7));
        // An error arises before the third expression is produced (identifier `_`), since the tokenizer 
        // generates the next token within the parser as the previous expression is parsed
        // This tokenizer error bubbles up to the parser, preventing the next expression 
        // from being returned, in favour of the error
        expect_error(&tree[2]);
    }

    #[test] 
    fn octal() {
        let tree = generate_tree("0o0;0o5;_ 0o9;");
        assert_eq!(tree[0], ok_tree(AST::Number { value: 0.0 }, 0..3));
        assert_eq!(tree[1], ok_tree(AST::Number { value: 5.0 }, 4..7));
        // Same reason as above
        expect_error(&tree[2]);
    }
}