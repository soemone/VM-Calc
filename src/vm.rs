use std::collections::HashMap;

use crate::{ast::Operator, functions::get_function, instruction::{self, Instruction, Value}};

pub enum VMError {
    BinOnNaN,
    InvalidBytecode,
    ErrString(String)
}

pub struct VM<'a> {
    instructions: Vec<Instruction<'a>>,
    stack: Vec<Value>,
    pc: usize,
    symbols: HashMap<&'a str, Value>
}

impl<'a> VM<'a> {
    pub fn new(instructions: Vec<Instruction<'a>>) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            symbols: HashMap::new(),
            instructions,
        }
    }

    pub fn execute_all(&mut self) {
        while self.pc < self.instructions.len() {
            match self.execute_next() {
                Ok(_) => (),
                Err(error) => {
                    // Stop the vm since a runtime error has occured.
                    self.pc = self.instructions.len();
                    match error {
                        VMError::BinOnNaN => println!("[Runtime Error]: Binary operation cannot be performed on a value that is not a number"),
                        VMError::InvalidBytecode => println!("[Runtime Error]: The bytecode provided to the VM appears to be invalid, or containing a bug that causes the program to unexpectedly crash"),
                        VMError::ErrString(string) => println!("[Runtime Error]: {string}"),
                    }
                }
            };
        }
    }

    pub fn execute_next(&mut self) -> Result<(), VMError> {
        self.pc += 1;
        match &self.instructions[self.pc - 1] {
            Instruction::Load { value } => {
                self.stack.push(*value)
            },

            Instruction::Binary { operator } => {
                let rhs = match self.stack.pop().unwrap() {
                    Value::Number(number) => number,
                    _ => return Err(VMError::BinOnNaN),
                };
                let lhs = match self.stack.pop().unwrap() {
                    Value::Number(number) => number,
                    _ => return Err(VMError::BinOnNaN),
                };
                let result = match operator {
                    Operator::Plus => lhs + rhs,
                    Operator::Minus => lhs - rhs,
                    Operator::Multiply => lhs * rhs,
                    Operator::Divide => {
                        if rhs == 0.0 {
                            return Err(VMError::ErrString(format!("Cannot divide a number by zero!")));
                        }
                        lhs / rhs
                    },
                    Operator::Exponent => lhs.powf(rhs),
                    Operator::BitAnd => (lhs as usize & rhs as usize) as f64,
                    Operator::BitOr => (lhs as usize | rhs as usize) as f64,
                    Operator::BitXor => (lhs as usize ^ rhs as usize) as f64,
                    Operator::BitLeftShift => ((lhs as usize) << (rhs as usize)) as f64,
                    Operator::BitRightShift => ((lhs as usize) >> (rhs as usize)) as f64,
                };
                self.stack.push(Value::Number(result));
            },

            Instruction::Unary { operator } => {
                let rhs = match self.stack.pop().unwrap() {
                    Value::Number(number) => number,
                    _ => return Err(VMError::ErrString(format!("Cannot perform unary operations on non numerical values"))),
                };

                let result = match operator {
                    Operator::Plus => rhs,
                    Operator::Minus => -rhs,
                    _ => return Err(VMError::ErrString(format!("Unable to perform unary operation {operator} on a number!"))),
                };
                self.stack.push(Value::Number(result))
            }

            Instruction::Output => {
                let res = match self.stack.pop() {
                    Some(res) => res,
                    None => return Err(VMError::InvalidBytecode), 
                };
                println!("Result: {}", res)
            },

            Instruction::LoadSymbolName { name } => {
                self.symbols.insert(name, Value::Null);
                self.stack.push(Value::Null);
            },

            Instruction::LoadSymbol { name } => {
                let value = match self.stack.pop() {
                    Some(res) => res,
                    None => return Err(VMError::InvalidBytecode), 
                };
                self.symbols.insert(name, value);
                self.stack.push(Value::Null);
            },

            Instruction::CallSymbol { name } => {
                match self.symbols.get(name) {
                    Some(value) => self.stack.push(*value),
                    None => return Err(VMError::ErrString(format!("The variable {name} does not exist!"))),
                }
            },

            Instruction::ReloadSymbol { name } => {
                match self.symbols.get_mut(name) {
                    Some(value) => {
                        let new_value = match self.stack.pop() {
                            Some(res) => res,
                            None => return Err(VMError::InvalidBytecode), 
                        };        
                        *value = new_value;
                    },
                    None => return Err(VMError::ErrString(format!("Cannot assign a value to variable {name} because it does not exist!"))),
                }
                self.stack.push(Value::Null);
            },

            Instruction::FunctionCall { name } => {
                let mut arguments = vec![];
                let (length, function) = get_function(name).unwrap();
                for _ in 0..length {
                    let arg = match self.stack.pop() {
                        Some(value) => {
                            match value {
                                Value::Number(num) => num,
                                _ => return Err(VMError::ErrString(format!("Functions that do not deal with values other than numbers are not yet supported!"))),
                            }
                        },
                        None => return Err(VMError::ErrString(format!("Failed to get arguments to function {name} (Likely an internal error)!"))),
                    };
                    arguments.push(arg);
                }
                self.stack.push(Value::Number(function(arguments.as_slice())))
            }

            _ => unimplemented!(),
        };
        Ok(())
    }
}