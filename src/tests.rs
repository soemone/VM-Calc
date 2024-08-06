use crate::lexer;
#[cfg(test)]

// These are just tests. Nothing to see here, that is if physical laws are still the same. I hope they are, at least.
// If not, this place is gonna need renovation.
mod tests {
    use crate::tokens::{NumberType, Token, TokenType, TokenizerError};
    use super::*;

    #[test]
    fn test_numbers() -> Result<(), ()>{
        let mut lexer = lexer::Lexer::new("1 2.4 08 5. 0o 0b 0x 0b110 0o01234567 0x0123456789ABCDEFabcdef")?;
        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: 1 }, 0))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: 3 }, 2))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: 2 }, 6))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: 2 }, 9))
        );

        assert_eq!(
            lexer.next(), 
            Err(TokenizerError::NumberExpected),
        );

        assert_eq!(
            lexer.next(), 
            Err(TokenizerError::NumberExpected),
        );

        assert_eq!(
            lexer.next(), 
            Err(TokenizerError::NumberExpected),
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Binary, length: 5 }, 21))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Octal, length: 10 }, 27))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Number { number_type: NumberType::Hex, length: 24 }, 38))
        );

        Ok(())
    }

    #[test]
    fn test_identifiers() -> Result<(), ()>{
        let mut lexer = lexer::Lexer::new("a ab abc a~b")?;
        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Identifier { length: 1 }, 0))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Identifier { length: 2 }, 2))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Identifier { length: 3 }, 5))
        );

        assert_eq!(
            lexer.next(), 
            Ok(Token::new(TokenType::Identifier { length: 3 }, 9))
        );
        Ok(())
    }

    #[test]
    fn test_operators_and_delimiters() -> Result<(), ()>{
        let mut lexer = lexer::Lexer::new("* *= ** **= + += - -= = [ ( ) ] ; / /= << <<= >> >>= & &= | |= ^ ^=")?;
        use TokenType::*;
        let list = 
            [
                (Multiply, 1), 
                (MultiplyEqual, 2),
                (Exponent, 2),
                (ExponentEqual, 3),
                (Add, 1),
                (AddEqual, 2),
                (Subtract, 1),
                (SubtractEqual, 2),
                (Equal, 1),
                (OpeningBracket, 1),
                (OpeningBracket, 1),
                (ClosingBracket, 1),
                (ClosingBracket, 1),
                (Semicolon, 1),
                (Divide, 1),
                (DivideEqual, 2),
                (BitLeftShift, 2),
                (BitLeftShiftEqual, 3),
                (BitRightShift, 2),
                (BitRightShiftEqual, 3),
                (BitAnd, 1),
                (BitAndEqual, 2),
                (BitOr, 1),
                (BitOrEqual, 2),
                (BitXor, 1),
                (BitXorEqual, 2),
            ];
        let mut idx = 0;
        for (item, len) in list {
            assert_eq!(
                lexer.next(), 
                Ok(Token::new(item, idx))
            );
            idx += len + 1;
        }
        Ok(())
    }

    #[test]
    fn test_invalid() -> Result<(), ()>{
        let mut lexer = lexer::Lexer::new("` >")?;
        assert_eq!(
            lexer.next(), 
            Err(TokenizerError::InvalidCharacter)
        );

        assert_eq!(
            lexer.next(), 
            Err(TokenizerError::InvalidCharacter)
        );
        
        assert_eq!(
            lexer.next(), 
            Err(TokenizerError::EOF)
        );
        Ok(())
    }

}