
use std::{collections::HashMap, rc::Rc};
use crate::{ast::{Operator, Tree, AST}, errors::Error, functions::get_function, lexer::Lexer, tokens::{NumberType, Token, TokenType}, utils::Span};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    pub(crate) eof: bool,
    function_symbols: HashMap<&'a str, usize>,
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
            function_symbols: HashMap::new(),
        }
    }

    pub fn new_fn_symbols(lexer: Lexer<'a>, function_symbols: HashMap<&'a str, usize>) -> Self {
        Self {
            token: Token::null(),
            lexer,
            eof: false,
            function_symbols,
        }
    }

    pub fn get_fn_symbols(self) -> HashMap<&'a str, usize> {
        self.function_symbols
    }

    // Used only for tests
    pub fn generate_expressions(&mut self) -> Vec<Result<Rc<Tree<'a>>, Error>> {
        self.increment().ok();
        let mut expressions = vec![];
        while !self.eof {
            match self.expression() {
                Ok(ast) => {
                    expressions.push(Ok(ast))
                },
                Err(error) => {
                    expressions.push(Err(error))
                }
            }
        }
        expressions
    }

    pub fn next_expression(&mut self) -> Result<Rc<Tree<'a>>, Error> {
        if self.token.token_type == TokenType::Null {
            self.increment()?;
        }
        self.expression()
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
        let mut result = self.final_stage()?;
        match &self.token.token_type {
            TokenType::Semicolon => self.increment()?,
            TokenType::Colon => {
                self.increment()?;
                let span = result.span;
                result = Rc::new(Tree::new(AST::Output { value: result }, span));
            }
            _ => {
                let span = Span::new(self.token.span.start, self.token.span.start);
                return 
                    Err(Error::PError {
                        message: format!("Expected semicolon (`;`) or colon (`:`) after an expression! Found `{}`", &self.lexer.source[self.token.span.as_range()]), 
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

                        // A function declaration
                        TokenType::Identifier => {
                            if let Some(..) = self.function_symbols.get(name) {
                                return Err(Error::PError { 
                                        message: format!("The function `{name}` already exists!"), 
                                        span: identifier_span,
                                    });
                            } else if let Ok(..) = get_function(name) {
                                return Err(Error::PError { 
                                    message: format!("The function `{name}` is a built in function and cannot be overwritten!"), 
                                    span: identifier_span,
                                });
                            }
                            let mut arguments = vec![];
                            while self.token.token_type == TokenType::Identifier {
                                let name = &self.lexer.source[self.token.span.as_range()];
                                arguments.push(name);
                                self.increment()?;
                            }

                            if arguments.len() == 1 && arguments[0] == "_" {
                                arguments = vec![];
                            }

                            if self.token.token_type == TokenType::Equal {
                                self.increment()?;
                                let body = self.final_stage()?;
                                self.function_symbols.insert(name, arguments.len());
                                return Ok(Rc::new(
                                    Tree::new(
                                        AST::FunctionDecl { name, arguments, body },
                                        Span::new(start, self.token.span.end)
                                    )
                                ));
                            } else {
                                return Err(Error::PError 
                                    { 
                                        message: format!("Expected an expression for the function `{name}`"), 
                                        span: identifier_span,
                                    });
                            }
                        },

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
                } 
                else if name == "delete" {
                    // self.increment()?;
                    let value = &self.lexer.source[self.token.span.as_range()];
                    let token_span = self.token.span;
                    match &self.token.token_type {
                        TokenType::Identifier => {
                            self.increment()?;
                            self.function_symbols.remove(value);
                            Ok(Rc::new(
                                Tree::new(
                                    AST::Delete { name: value },
                                    Span::new(start, self.token.span.end)
                                )
                            ))
                        },

                        tt => 
                            Err(Error::PError { 
                                    message: format!("Expected an identifer / function to delete but found `{tt}`"), 
                                    span: token_span 
                                })
                    }
                }
                else if name == "Null" {
                    Ok(Rc::new(
                        Tree::new(
                            AST::Null,
                            Span::new(start, self.token.span.end)
                        )
                    ))
                } else {

                    let token = self.token.token_type.clone();

                    let mut assign_type = |operator| -> Result<Rc<Tree<'a>>, Error> {
                        self.increment()?;
                        let result = self.final_stage()?;
                        let end = result.span.end;
                        return Ok(Rc::new(
                            Tree::new(
                                AST::AssignOp { operator, identifier: name, identifier_span: Span::new(start, ident_end), value: result },
                                Span::new(start, end)
                            )
                        ));
                    };

                    match token {
                        // Change variable assignment
                        TokenType::Equal => {
                            self.increment()?;
                            let result = self.final_stage()?;
                            let end = result.span.end;
                            return Ok(Rc::new(
                                Tree::new(
                                    AST::Assign { identifier: name, identifier_span: Span::new(start, ident_end), value: result },
                                    Span::new(start, end)
                                )
                            ));
                        },

                        // Function Call
                        TokenType::OpeningBracket => {
                            self.increment()?;
                            let expr_start = self.token.span.start;
                            let mut expressions = vec![];
                            while self.token.token_type != TokenType::ClosingBracket {
                                expressions.push(self.final_stage()?);
                                if self.token.token_type == TokenType::ClosingBracket {
                                    break;
                                }
                                self.expect(TokenType::Comma)?;
                                self.increment()?;
                            }
                            self.increment()?;
                            let end = self.token.span.end - 1;
    
                            match get_function(name) {
                                Ok((arg_len, _)) => {
                                    if arg_len != expressions.len() {
                                        return Err(Error::PError { 
                                            message: format!("The function `{name}` expected {arg_len} argument(s) but {} argument(s) were found!", expressions.len()), 
                                            span: Span::new(expr_start, end - 1),
                                        })
                                    }
                                },
    
                                Err(()) => {
                                    if !self.function_symbols.contains_key(name) {
                                        return Err(Error::PError { 
                                            message: format!("The function `{name}` does not exist!"), 
                                            span: Span::new(start, end),
                                        });
                                    } else {
                                        let arg_len = self.function_symbols.get(name).unwrap();
                                        if expressions.len() != *arg_len {
                                            return Err(Error::PError { 
                                                message: format!("The function `{name}` expected {arg_len} argument(s) but {} argument(s) were found!", expressions.len()), 
                                                span: Span::new(expr_start, end - 1),
                                            });
                                        }
                                    }
                                }
                            };
    
                            return Ok(Rc::new(
                                Tree::new(
                                    AST::FunctionCall{ name, expressions },
                                    Span::new(start, end)
                                )
                            ));    
                        }
                        
                        TokenType::AddEqual => return assign_type(Operator::PlusEqual),
                        TokenType::SubtractEqual => return assign_type(Operator::MinusEqual),
                        TokenType::MultiplyEqual => return assign_type(Operator::MultiplyEqual),
                        TokenType::DivideEqual => return assign_type(Operator::DivideEqual),
                        TokenType::ExponentEqual => return assign_type(Operator::ExponentEqual),
                        TokenType::BitAndEqual => return assign_type(Operator::BitAndEqual),
                        TokenType::BitOrEqual => return assign_type(Operator::BitOrEqual),
                        TokenType::BitXorEqual => return assign_type(Operator::BitXorEqual),
                        TokenType::BitLeftShiftEqual => return assign_type(Operator::BitLeftShiftEqual),
                        TokenType::BitRightShiftEqual => return assign_type(Operator::BitRightShiftEqual),

                        _ => ()
                    };

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

            TokenType::EOF => Err(Error::NoResult),

            token => {
                self.increment()?;
                Err(Error::PInvalidStatement {
                    message: format!("An unexpected or invalid token `{}` was found", token),
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