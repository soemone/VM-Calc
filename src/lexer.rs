use std::str::Chars;

use crate::tokens::{NumberType, Token, TokenType, TokenizerError};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    position: usize,
}

impl Lexer<'_> {
    pub fn new(source: &'_ str) -> Result<Lexer, ()> {
        let chars = source.chars();
        Ok(Lexer {
            source,
            chars,
            current: None,
            position: 0,
        })
    }

    pub fn next(&mut self) -> Result<Token, TokenizerError>{
        let start = self.position;

        macro_rules! token {
            (TokenType::$token_type: ident) => {{
                self.increment();
                Ok(Token::new(TokenType::$token_type, start))
            }};
        }

        macro_rules! variable_token {
            ($number_of_increments: expr, TokenType::$token_type: ident) => {{
                for _ in 0..$number_of_increments {
                    self.increment();
                }
                Ok(Token::new(TokenType::$token_type, start))
            }};
        }


        let next =
            match self.chars.clone().next() {
                Some(character) => character,
                None => return Err(TokenizerError::EOF),
            };

        match next {
            // Skip whitespace
            _ if Self::check_whitespace(next) => {
                self.take_while(Self::check_whitespace);
                return self.next();
            },

            // Numbers
            _ if Self::check_number(next) => {
                if next == '0' {
                    self.increment();
                    let mut cloned_iter = self.chars.clone();
                    match cloned_iter.next() {
                        // Octal
                        Some('o') => {
                            self.increment();
                            self.take_while(Self::check_octal);
                            if self.position - start == 2 {
                                return Err(TokenizerError::NumberExpected);
                            }
                            Ok(Token::new(TokenType::Number { number_type: NumberType::Octal, length: self.position - start }, start))
                        },

                        // Binary
                        Some('b') => {
                            self.increment();
                            self.take_while(Self::check_binary);
                            if self.position - start == 2 {
                                return Err(TokenizerError::NumberExpected);
                            }
                            Ok(Token::new(TokenType::Number { number_type: NumberType::Binary, length: self.position - start }, start))
                        }

                        // Hex
                        Some('x') => {
                            self.increment();
                            self.take_while(Self::check_hex);
                            if self.position - start == 2 {
                                return Err(TokenizerError::NumberExpected);
                            }
                            Ok(Token::new(TokenType::Number { number_type: NumberType::Hex, length: self.position - start }, start))
                        }

                        // A number that starts with zero. Why? Why not.
                        Some(character) if Self::check_number(character) => {
                            // Parse number
                            self.take_while(Self::check_number);
                            let mut cloned_iter = self.chars.clone();
                            if let Some('.') = cloned_iter.next() {
                                self.increment();
                            }
                            self.take_while(Self::check_number);
                            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: self.position - start }, start))
                        }

                        // Floating point expression
                        Some('.') => {
                            match cloned_iter.next() {
                                Some(character) if Self::check_number(character) => {
                                    self.take_while(Self::check_number);
                                }
                                _ => ()
                            }
                            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: self.position - start }, start))
                        }
                        _ => {
                            // Just a zero
                            Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: self.position - start }, start))
                        }
                    }
                } else {
                    // Take the rest of the numbers
                    self.take_while(Self::check_number);
                    let mut cloned_iter = self.chars.clone();
                    if let Some('.') = cloned_iter.next() {
                        self.increment();
                        self.take_while(Self::check_number);
                    }
                    Ok(Token::new(TokenType::Number { number_type: NumberType::Real, length: self.position - start }, start))
                }
            },

            // Register identifiers
            _ if Self::check_ident_start(next) => {
                self.take_while(Self::check_ident_continue);
                Ok(Token::new(TokenType::Identifier { length: self.position - start }, start))
            },


            // Delimiters 
            '(' | '[' => token!(TokenType::OpeningBracket),

            ')' | ']' => token!(TokenType::ClosingBracket),

            ';' => token!(TokenType::Semicolon),

            // Operators
            '=' => token!(TokenType::Equal),

            '+' => {
                self.increment();
                match self.peek() {
                    Ok('=') => variable_token!(1, TokenType::AddEqual),
                    _ => variable_token!(0, TokenType::Add)
                }
            },

            '-' => {
                self.increment();
                match self.peek() {
                    Ok('=') => variable_token!(1, TokenType::SubtractEqual),
                    _ => variable_token!(0, TokenType::Subtract)
                }
            },

            '/' => {
                self.increment();
                match self.peek() {
                    Ok('=') => token!(TokenType::DivideEqual),
                    // Get rid of comments
                    Ok('/') => {
                        self.take_while(|character| character != '\n');
                        return self.next();
                    }
                    _ => variable_token!(0, TokenType::Divide)
                }
            },

            '*' => {
                self.increment();
                match self.peek() {
                    Ok('=') => token!(TokenType::MultiplyEqual),
                    Ok('*') => {
                        self.increment();
                        match self.peek() {
                            Ok('=') => token!(TokenType::ExponentEqual),
                            _ => variable_token!(0, TokenType::Exponent)
                        }
                    }
                    _ => variable_token!(0, TokenType::Multiply)
                }
            },

            '<' => {
                self.increment();
                match self.peek() {
                    Ok('<') => {
                        self.increment();
                        match self.peek() {
                            Ok('=') => token!(TokenType::BitLeftShiftEqual),
                            _ => variable_token!(0, TokenType::BitLeftShift)
                        }
                    }
                    _ => Err(TokenizerError::InvalidCharacter)
                }
            },

            '>' => {
                self.increment();
                match self.peek() {
                    Ok('>') => {
                        self.increment();
                        match self.peek() {
                            Ok('=') => token!(TokenType::BitRightShiftEqual),
                            _ => variable_token!(0, TokenType::BitRightShift)
                        }
                    }
                    _ => Err(TokenizerError::InvalidCharacter)
                }
            },

            '&' => {
                self.increment();
                match self.peek() {
                    Ok('=') => variable_token!(1, TokenType::BitAndEqual),
                    _ => variable_token!(0, TokenType::BitAnd)
                }
            },

            '|' => {
                self.increment();
                match self.peek() {
                    Ok('=') => variable_token!(1, TokenType::BitOrEqual),
                    _ => variable_token!(0, TokenType::BitOr)
                }
            },

            '^' => {
                self.increment();
                match self.peek() {
                    Ok('=') => variable_token!(1, TokenType::BitXorEqual),
                    _ => variable_token!(0, TokenType::BitXor)
                }
            },

            // Invalid characters
            _ => {
                println!("Unrecognized character @ {}", start);
                self.increment();
                Err(TokenizerError::InvalidCharacter)
            },
        }
    }

    // utils

    fn take_while<T>(&mut self, mut predicate: T) 
        where T: FnMut(char) -> bool {
            let mut cloned_iter = self.chars.clone();
            loop {
                match cloned_iter.next() {
                    Some(character) if predicate(character) => {
                        self.chars.next();
                        self.current = Some(character);
                        self.position += character.len_utf8();
                    },
                    _ => break,
                };
            }
    }

    fn get_next(&mut self) -> Result<char, ()> {
        let character = 
            self
                .chars
                .next()
                .ok_or(())?;
        self.position += character.len_utf8();
        self.current = Some(character);
        Ok(character)
    }

    fn get_next_unchecked(&mut self) -> char {
        let character = 
            self
                .chars
                .next()
                .unwrap();
        self.position += character.len_utf8();
        self.current = Some(character);
        character
    }

    fn increment(&mut self) {
        let character = 
            self
                .chars
                .next()
                .unwrap_or('\0');
        self.position += character.len_utf8();
        self.current = Some(character);
    }

    fn peek(&self) -> Result<char, ()> {
        let mut cloned_iter = self.chars.clone();
        cloned_iter.next().ok_or(())
    }

    fn check_ident_start(character: char) -> bool {
        matches!(character, 'A'..='Z' | 'a'..='z' | '_' | '~')
    }

    fn check_ident_continue(character: char) -> bool {
        matches!(character, 'A'..='Z' | 'a'..='z' | '_' | '~' | '0'..='9')
    }

    fn check_whitespace(character: char) -> bool {
        char::is_whitespace(character)
    }

    fn check_number(character: char) -> bool {
        matches!(character, '0'..='9')
    }

    fn check_octal(character: char) -> bool {
        matches!(character, '0'..='7')
    }
    
    fn check_binary(character: char) -> bool {
        matches!(character, '0'..='1')
    }

    fn check_hex(character: char) -> bool {
        matches!(character, '0'..='9' | 'A'..='F' | 'a'..='f')
    }
}