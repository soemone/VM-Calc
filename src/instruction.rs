use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::ast::Operator;


// There most definitely is a better, more efficient way to represent the bytecode, but I cannot think of it
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Value::Number(number) => format!("{number}"),
            // WHY?
            Value::Null => format!("{}NULL{}", "{", "}"),
        };
        write!(f, "{res}")
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(align(1))]
pub enum Instruction<'a> {
    /// Load a value into the stack
    Load {
        value: Value,
    },

    /// Perform a binary operation
    Binary {
        operator: Operator,
    },

    /// Perform a unary operation
    Unary {
        operator: Operator,
    },

    /// Create a variable and initialize it with a null value
    LoadSymbolName {
        name: &'a str,
    },

    /// Create a variable and initialize it with a given value
    LoadSymbol {
        name: &'a str,
    },

    /// Change the value of a variable
    ReloadSymbol {
        name: &'a str,
    },

    /// Invoke the value of a variable
    CallSymbol {
        name: &'a str,
    },

    /// Invoke a function
    FunctionCall {
        name: &'a str,
    },

    /// Present the result of the previous expression to the terminal
    Output,

    /// An illegal instruction
    Illegal,
}