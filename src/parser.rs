
use std::{rc::Rc, result};

use crate::{ast::{Tree, AST}, errors::Error, lexer::Lexer, tokens::{NumberType, Token, TokenType}, utils::Span};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    eof: bool,
}
macro_rules! create_fn {
    ($self: ident, $below_fn: ident, $token_type: pat) => {{
        let mut result = $self.$below_fn()?;
        while matches!(&$self.token.token_type, $token_type) && !$self.eof {
            let operator = $self.token.token_type.clone().into();
            $self.increment()?;
            if $self.eof {
                return Err(Error::PError { 
                    message: format!("Expected an expression after the `{}` operator, but found nothing. @ {}", $self.token.token_type, &$self.token.span), 
                    span: $self.token.span
                });
            } else {
                let rhs = $self.$below_fn()?;
                let end = rhs.span.end;
                result = 
                    Rc::new(
                        Tree::new(
                            AST::BinaryOp { lhs: Rc::clone(&result), rhs, op: operator }, 
                            Span::new(result.span.start, end)
                        )
                    )
            }
        }
        Ok(result)
    }}
}

impl<'a> Parser<'a> {

    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            token: Token::null(),
            lexer,
            eof: false,
        }
    }

    pub fn generate_expressions(&mut self) {
        self.increment().ok();
        while !self.eof {
            match self.expression() {
                Ok(ast) => {
                    println!("{ast}")
                },
                Err(error) => {
                    println!("{error}");
                }
            }
        }
    }

    pub fn increment(&mut self) -> Result<(), Error>{
        match self.lexer.next() {
            Ok(token) => {
                self.token = token;
                Ok(())
            },
            // Check if we're at the end of the file.
            Err(Error::TEOF) => {
                self.eof = true;
                self.token = Token::eof(self.token.span.end);
                Ok(())
            },
            Err(error) => Err(error)
        }
    }

    fn expression(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        let result = self.final_stage()?;
        match &self.token.token_type {
            TokenType::Semicolon => self.increment()?,
            _ => {
                let span = Span::new(self.token.span.start, self.token.span.start);
                return 
                    Err(Error::PError { 
                        message: format!("Expected semicolon after an expression! Found `{}` @ {}", &self.token.token_type, span), 
                        span,
                    });
            },
        }
        Ok(result)
    }

    fn final_stage(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        self.bitor()
    }

    fn bitor(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, bitxor, TokenType::BitOr)
    }

    fn bitxor(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, bitand, TokenType::BitXor)
    }

    fn bitand(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, bitshift, TokenType::BitAnd)
    }

    fn bitshift(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, term, TokenType::BitLeftShift | TokenType::BitRightShift)
    }

    fn term(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, factor, TokenType::Add | TokenType::Subtract)
    }

    fn factor(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, exponent, TokenType::Multiply | TokenType::Divide)
    }

    fn exponent(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        create_fn!(self, unary, TokenType::Exponent)
    }

    fn unary(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        let start = self.token.span.start;
        match &self.token.token_type {
            TokenType::Add | TokenType::Subtract => {
                let operator = self.token.token_type.clone().into();
                self.increment()?;
                if self.eof {
                    return Err(Error::PError { 
                        message: format!("Expected an expression after the `{}` operator, but found nothing. @ {}", &self.token.token_type, &self.token.span), 
                        span: self.token.span
                    });
                } else {
                    let rhs = self.unary()?;
                    let end = rhs.span.end;
                    Ok(Rc::new(
                        Tree::new(
                            AST::UnaryOp { rhs, op: operator },
                            Span::new(start, end)
                        )
                    ))
                }
            }

            _ => self.base()
        }
    }

    fn base(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        let span = self.token.span;
        match self.token.token_type.clone() {
            TokenType::Number { number_type } => {
                self.increment()?;
                match number_type {
                    NumberType::Binary => self.parse_number(span, 2, "Binary"),

                    NumberType::Octal => self.parse_number(span, 8, "Octal"),

                    NumberType::Hex => self.parse_number(span, 16, "Hexadecimal"),

                    NumberType::Real => {
                        let real_str = &self.lexer.source[span.as_range()];
                        let real_number: f64 = 
                            match real_str.parse() {
                                Ok(value) => value,
                                // This too.
                                Err(parse_error) => {
                                    return Err(Error::PInternalError { 
                                        message: format!("Real number parse error @ {span}. Message: {parse_error:?}"), 
                                        span,
                                    });
                                }
                            };
                        let number_ast = AST::Number {
                            value: real_number,
                        };
                        Ok(Rc::new(Tree::new(number_ast, span)))
                    },
                }
            }

            TokenType::Identifier => {
                let name = &self.lexer.source[self.token.span.as_range()];
                let start = self.token.span.start;
                let ident_end = self.token.span.end;
                self.increment()?;
                if name == "let" {
                    self.expect(TokenType::Identifier)?;
                    let identifier_span = self.token.span;
                    let name = &self.lexer.source[self.token.span.as_range()];
                    self.increment()?;
                    match self.token.token_type {
                        // Declare a variable while assigning a value to it
                        TokenType::Equal => {
                            self.increment()?;
                            let result = self.final_stage()?;
                            let end = result.span.end;
                            Ok(Rc::new(
                                Tree::new(
                                    AST::DeclareAssign { identifier: name, identifier_span, value: result },
                                    Span::new(start, end)
                                )
                            ))
                        }

                        // Just declare a variable
                        _ => {
                            let end = self.token.span.end;
                            Ok(Rc::new(
                                Tree::new(
                                    AST::Declare { identifier: name, identifier_span },
                                    Span::new(start, end)
                                )
                            ))
                        }
                    }
                } else {
                    // Change assignment of variable
                    if self.token.token_type == TokenType::Equal {
                        self.increment()?;
                        let result = self.final_stage()?;
                        let end = result.span.end;
                        return Ok(Rc::new(
                            Tree::new(
                                AST::Assign { identifier: name, identifier_span: Span::new(start, ident_end), value: result },
                                Span::new(start, end)
                            )
                        ));
                    }
                    let end = self.token.span.end;
                    // An identifier
                    Ok(Rc::new(
                        Tree::new(
                            AST::Identifier { name },
                            Span::new(start, end)
                        )
                    ))
                }
            }

            TokenType::OpeningBracket => {
                self.increment()?;
                let result = self.final_stage()?;
                match self.token.token_type {
                    TokenType::ClosingBracket => self.increment()?,
                     
                    _ => {
                        let span = Span::new(self.token.span.start, self.token.span.start);
                        return 
                            Err(Error::PError { 
                                message: format!("Expected closing bracket `) | ]`! Found `{}` @ {}", &self.token.token_type, span), 
                                span,
                            });
                    }
                };
                Ok(result)
            }

            _ => {
                self.increment()?;
                Err(Error::PInvalidStatement {
                    message: format!("An unexpected or invalid token was found @ {span}"),
                    span,
                })
            }
        }
    }
    
    fn expect(&mut self, token_type: TokenType) -> Result<(), Error> {
        if self.token.token_type != token_type {
            return Err(Error::PError { 
                message: format!("Expected a token of type: {token_type} but found token of type: {} @ {}", self.token.token_type, self.token.span), 
                span: self.token.span,
            })
        }
        Ok(())
    } 

    fn parse_number(&mut self, span: Span, radix: u32, number_type: &str) -> Result<Rc<Tree<'a>>, Error> {
        let binary_str = &self.lexer.source[(span.start + 2)..span.end];
        let number = 
            match u64::from_str_radix(binary_str, radix) {
                Ok(value) => value,
                // This *should* never trigger, but here it is
                Err(parse_error) => {
                    return Err(Error::PInternalError { 
                        message: format!("{number_type} number parse error @ {span}. Message: {parse_error:?}"), 
                        span,
                    });
                }
            } as f64;
        let number_ast = AST::Number {
            value: number,
        };
        Ok(Rc::new(Tree::new(number_ast, span)))
    }
}