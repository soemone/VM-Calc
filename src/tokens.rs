use std::fmt::Debug;


#[derive(Debug, PartialEq)]
pub enum NumberType {
    /// 0b1001
    Binary,
    /// 1, 1.001, 0.0000001, .1111, 1.
    Real,
    /// 0o1113
    Octal,
    /// 0xFFF
    Hex,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Basic elements
    /// Numbers of any type
    Number {
        number_type: NumberType,
        length: usize,
    },
    /// Variables or functions
    Identifier {
        length: usize,
    },

    // Delimiters
    /// Semicolon to seperate statements
    Semicolon,
    /// [, (
    OpeningBracket,
    /// ], (
    ClosingBracket,

    // Operators
    /// *
    Multiply,
    /// *=
    MultiplyEqual,
    
    /// /
    Divide,
    /// /=
    DivideEqual,    

    /// +
    Add,
    /// +=
    AddEqual,
    
    /// -
    Subtract,
    /// -=
    SubtractEqual,

    /// **
    Exponent,
    /// **=
    ExponentEqual,

    /// ^
    BitXor,
    /// ^=
    BitXorEqual,

    /// &
    BitAnd,
    /// &=
    BitAndEqual,

    /// |
    BitOr,
    /// |=
    BitOrEqual,

    /// <<
    BitLeftShift,
    /// <<=
    BitLeftShiftEqual,

    /// >>
    BitRightShift,
    /// >>=
    BitRightShiftEqual,

    /// =
    Equal,
}

// This technically is not an error enum if it has `EOF` in it.
// But I have no idea what else to call it
#[derive(Debug, PartialEq)]
pub enum TokenizerError {
    /// An error that occurs when an octal, binary or hexadecimal number provided is incomplete - `0x` `0b` `0o`
    NumberExpected,
    /// An error that occurs when an invalid character is encountered
    InvalidCharacter,
    /// End of file
    EOF,
}

#[derive(PartialEq)]
pub struct Token {
    token_type: TokenType,
    position: usize,
}

impl Token {
    pub fn new(token_type: TokenType, position: usize) -> Token {
        Token { token_type, position }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} type @ {}", self.token_type, self.position)
    }
}