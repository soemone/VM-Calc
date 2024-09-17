use std::{ops::Range, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::ast::Operator;

// There most definitely is a better, more efficient way to represent the bytecode, but I cannot think of it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match &self {
            Value::Number(number) => format!("{number}"),
            Value::String(string) => {format!("{}", string)},
            // WHY?
            Value::Null => format!("{}NULL{}", "{", "}"),
        };
        write!(f, "{res}")
    }
}


impl Value {
    pub fn type_of(&self) -> &str {
        match self {
            Value::Null => "{Null}",
            Value::Number(..) => "{Number}",
            Value::String(..) => "{String}",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

    /// Change the value of a variable
    ReloadSymbolOp {
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

    FunctionDecl {
        name: &'a str,
    },

    ArgumentName {
        name: &'a str,
    },

    Delete {
        name: &'a str,
    },

    Print {
        depth: usize
    },

    UData { number: usize },

    OData { operator: Operator },

    /// A null value
    Null,

    /// Present the result of the previous expression to the terminal
    Output,

    /// A flag to not run the VM when a compiler error has occured
    CompileError,

    // Not needed
    // /// An illegal instruction
    // Illegal,
}

pub struct Function {
    pub(crate) arguments: usize,
    pub(crate) instructions: Range<usize>,
}

impl Function {
    pub fn new(arguments: usize, instructions: Range<usize>) -> Self {
        Self { arguments, instructions }
    }
}