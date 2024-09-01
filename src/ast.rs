use std::{fmt::Display, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{tokens::TokenType, utils::Span};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    Plus,
    PlusEqual,
    Minus,
    MinusEqual,
    Divide,
    DivideEqual,
    Multiply,
    MultiplyEqual,
    Exponent,
    ExponentEqual,
    BitAnd,
    BitOr,
    BitXor,
    BitLeftShift,
    BitRightShift,
    BitAndEqual,
    BitOrEqual,
    BitXorEqual,
    BitLeftShiftEqual,
    BitRightShiftEqual,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Exponent => "**",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::BitRightShift => ">>",
            Self::BitLeftShift => "<<",
            Self::PlusEqual => "+=",
            Self::MinusEqual => "-=",
            Self::DivideEqual => "/=",
            Self::MultiplyEqual => "*=",
            Self::ExponentEqual => "**=",
            Self::BitAndEqual => "&=",
            Self::BitOrEqual => "|=",
            Self::BitXorEqual => "^=",
            Self::BitLeftShiftEqual => "<<=",
            Self::BitRightShiftEqual => ">>=",
        };
        write!(f, "{res}")
    }
}

impl From<TokenType> for Operator {    
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Add => Self::Plus,
            TokenType::AddEqual => Self::Plus,
            TokenType::Subtract => Self::Minus,
            TokenType::SubtractEqual => Self::MinusEqual,
            TokenType::Divide => Self::Divide,
            TokenType::DivideEqual => Self::DivideEqual,
            TokenType::Multiply => Self::Multiply,
            TokenType::MultiplyEqual => Self::MultiplyEqual,
            TokenType::Exponent => Self::Exponent,
            TokenType::ExponentEqual => Self::ExponentEqual,
            TokenType::BitAnd => Self::BitAnd,
            TokenType::BitXor => Self::BitXor,
            TokenType::BitOr => Self::BitOr,
            TokenType::BitLeftShift => Self::BitLeftShift,
            TokenType::BitRightShift => Self::BitRightShift,

            _ => panic!("A bug has occured when trying to convert `{value:?}` to `Operator`"),
        }
    } 
}

#[derive(Debug, PartialEq)]
pub struct Tree<'a> {
    pub(crate) ast: AST<'a>,
    pub(crate) span: Span,
}

impl<'a> Tree<'a> {
    pub fn new(ast: AST<'a>, span: Span) -> Self {
        Self {
            ast, span,
        }
    }
}

impl Display for Tree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ast)
    }
}

#[derive(Debug, PartialEq)]
pub enum AST<'a> {
    BinaryOp { 
        lhs: Rc<Tree<'a>>,
        rhs: Rc<Tree<'a>>,
        op: Operator,
    },

    UnaryOp { 
        rhs: Rc<Tree<'a>>,
        op: Operator,
    },

    Number {
        value: f64,
    },

    Identifier {
        name: &'a str,
    },

    DeclareAssign {
        identifier: &'a str,
        identifier_span: Span,
        value: Rc<Tree<'a>>,
    },

    Declare {
        identifier: &'a str,
        identifier_span: Span,
    },

    Assign {
        identifier: &'a str,
        identifier_span: Span,
        value: Rc<Tree<'a>>,
    },

    AssignOp {
        identifier: &'a str,
        identifier_span: Span,
        value: Rc<Tree<'a>>,
        operator: Operator,
    },

    Output {
        value: Rc<Tree<'a>>,
    },

    FunctionCall {
        name: &'a str,
        expressions: Vec<Rc<Tree<'a>>>,
    },

    FunctionDecl {
        name: &'a str,
        arguments: Vec<&'a str>,
        body: Rc<Tree<'a>>,
    },

    Delete {
        name: &'a str,
    },

    Null,

    Invalid,
}

impl Display for AST<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "{}Null{}", "{", "}"),
            Self::BinaryOp { lhs, rhs, op } => write!(f, "({lhs} {op} {rhs})"),
            Self::UnaryOp { rhs, op } => write!(f, "({op}{rhs})"),
            Self::Number { value } => write!(f, "{value}"),
            Self::Identifier { name } => write!(f, "{name}"),
            Self::Assign { identifier, value, identifier_span: _} => write!(f, "({identifier} = {value})"),
            Self::AssignOp { operator, identifier, value, identifier_span: _} => write!(f, "({identifier} {operator}= {value})"),
            Self::Declare { identifier, identifier_span: _ } =>  write!(f, "(let {identifier})"),
            Self::DeclareAssign { identifier, value, identifier_span: _ } => write!(f, "(let {identifier} = {value})"),
            Self::Output { value } => write!(f, "*{value}*"),
            Self::FunctionDecl { name, arguments, body } => {
                write!(f, "let {name} {} = {body}", arguments.join(" "))
            }
            Self::Delete { name } => write!(f, "(delete {name})"),
            Self::FunctionCall { name, expressions } => {
                let mut arguments = String::new();
                for expr in expressions {
                    if arguments.is_empty() {
                        arguments = format!("{expr}");
                    } else {
                        arguments = format!("{arguments}, {expr}");
                    }
                }
                write!(f, "{name}({})", arguments)
            }
            Self::Invalid =>  write!(f, "(Invalid)"),
        }
    }
}