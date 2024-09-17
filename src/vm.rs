use std::{collections::HashMap, io::Write};

use serde::de::value;

use crate::{ast::Operator, functions::get_function, instruction::{Function, Instruction, Value}};

pub enum VMError {
    BinOnNaN,
    InvalidBytecode,
    ErrString(String)
}

pub struct VM<'a> {
    instructions: Vec<Instruction<'a>>,
    stack: Vec<Value>,
    pc: usize,
    pub(crate) outputs: Vec<Value>,
    symbols: HashMap<&'a str, Value>,
    functions: HashMap<&'a str, Function>,
}

impl<'a> VM<'a> {
    pub fn new(instructions: Vec<Instruction<'a>>) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            outputs: vec![],
            symbols: HashMap::new(),
            functions: HashMap::new(),
            instructions,
        }
    }

    pub fn new_with_symbols(instructions: Vec<Instruction<'a>>, symbols: HashMap<&'a str, Value>, functions: HashMap<&'a str, Function>) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            outputs: vec![],
            symbols,
            functions,
            instructions,
        }
    }

    pub fn print_output(&self) {
        if self.outputs.len() > 0 {
            println!("Results: {}", self.outputs.iter().map(|value| format!("{value}")).collect::<Vec<_>>().join(", "));
        } else {
            println!("No results for this expression");
        }
    }

    pub fn execute_all(&mut self) {
        // Don't run code that is empty or invalid
        if self.instructions.len() == 0 || self.instructions[0] == Instruction::CompileError {
            return;
        }

        while self.pc < self.instructions.len() {
            match self.execute_next() {
                Ok(_) => (),
                Err(error) => {
                    // Stop the vm since a runtime error has occured.
                    self.pc = self.instructions.len();
                    match error {
                        VMError::BinOnNaN => println!("[RUNTIME ERROR]: Binary operation cannot be performed on a value that is not a number"),
                        VMError::InvalidBytecode => println!("[RUNTIME ERROR]: The bytecode provided to the VM appears to be invalid, or containing a bug that causes the program to unexpectedly crash"),
                        VMError::ErrString(string) => println!("[RUNTIME ERROR]: {string}"),
                    }
                }
            };
        }
    }

    pub fn execute_next(&mut self) -> Result<(), VMError> {
        self.pc += 1;
        match &self.instructions[self.pc - 1] {
            Instruction::Load { value } => self.stack.push(value.clone()),

            Instruction::Binary { operator } => {
                let rhs = self.stack.pop().unwrap();
                let lhs = self.stack.pop().unwrap();
                match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => {
                        let res = match operator {
                            Operator::Plus => a + b,
                            Operator::Minus => a - b,
                            Operator::Multiply => a * b,
                            Operator::Divide => {
                                if b == 0.0 {
                                    return Err(VMError::ErrString(format!("Cannot divide a number by zero!")));
                                }
                                a / b
                            },
                            Operator::Exponent => a.powf(b),
                            Operator::BitAnd => (a as usize & b as usize) as f64,
                            Operator::BitOr => (a as usize | b as usize) as f64,
                            Operator::BitXor => (a as usize ^ b as usize) as f64,
                            Operator::BitLeftShift => ((a as usize) << (b as usize)) as f64,
                            Operator::BitRightShift => ((a as usize) >> (b as usize)) as f64,
                            _ => unimplemented!()
                        };
                        self.stack.push(Value::Number(res));
                    }

                    (Value::String(a), Value::String(b)) => {
                        let res = match operator {
                            Operator::Plus => {
                                let mut base = a;
                                base.push_str(&b);
                                base
                            },
                            _ => return Err(VMError::ErrString(format!("Cannot perform binary operation `{operator}` on strings!")))
                        };
                        self.stack.push(Value::String(res));
                    }

                    (a, b) => {
                        return Err(
                            VMError::ErrString(
                                format!(
                                    "Cannot perform binary operation `{operator}` on mismatched types: lhs `{}` and rhs `{}`!", 
                                    a.type_of(), b.type_of()
                                )
                            )
                        );
                    }
                }
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
                self.outputs.push(res)
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
                    Some(value) => self.stack.push(value.clone()),
                    None => return Err(VMError::ErrString(format!("The variable `{name}` does not exist!"))),
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

            Instruction::ReloadSymbolOp { name } => {
                let operator = match &self.instructions[self.pc] {
                    Instruction::OData { operator } => operator,
                    _ => return Err(VMError::InvalidBytecode),
                };

                self.pc += 1;

                match self.symbols.get_mut(name) {
                    Some(value) => {
                        let new_value = match self.stack.pop() {
                            Some(res) => res,
                            None => return Err(VMError::InvalidBytecode), 
                        };
                        match (new_value, value) {
                            (Value::Number(a), Value::Number(b)) => {
                                match operator {
                                    Operator::PlusEqual => *b += a,
                                    Operator::MinusEqual => *b -= a,
                                    Operator::DivideEqual => *b /= a,
                                    Operator::MultiplyEqual => *b *= a,
                                    Operator::ExponentEqual => *b = f64::powf(*b, a),
                                    Operator::BitOrEqual => *b = (*b as usize | a as usize) as f64,
                                    Operator::BitAndEqual => *b = (*b as usize & a as usize) as f64,
                                    Operator::BitXorEqual => *b = (*b as usize ^ a as usize) as f64,
                                    Operator::BitLeftShiftEqual => *b = ((*b as usize) << a as usize) as f64,
                                    Operator::BitRightShiftEqual => *b = (*b as usize >> a as usize) as f64,
        
                                    _ => unimplemented!()
                                };
                            }

                            (Value::String(a), Value::String(b)) => {
                                match operator {
                                    Operator::PlusEqual => b.push_str(a.as_str()),
        
                                    _ => return Err(VMError::ErrString(format!("Cannot perform operation `{operator}` on strings!"))),
                                };
                            }

                            (new_value, value) => {
                                return Err(
                                            VMError::ErrString(
                                                format!(
                                                    "Cannot perform operation `{operator}` on mismatched types: lhs `{}` and rhs `{}`!", 
                                                    value.type_of(), new_value.type_of()
                                                )
                                            )
                                        );
                            }
                        }
                    },
                    None => return Err(VMError::ErrString(format!("Cannot find variable {name} to change its value!"))),
                }
                self.stack.push(Value::Null);
            },

            // Really slow?
            Instruction::FunctionCall { name } => {
                let mut arguments = vec![];
                match get_function(name) {
                    Ok((length, function)) => {
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
                    },

                    // Look for function in function symbols
                    Err(..) => {
                        if let Some(function) = self.functions.get(name) {
                            let fn_args_address = function.instructions.start - function.arguments;
                            let fn_body_address = function.instructions.start;
                            let fn_body_end = function.instructions.end;

                            let orig_pc = self.pc;
                            let orig_symbols = self.symbols.clone();
                            self.pc = fn_args_address;
                            let args_len = fn_body_address - fn_args_address;
                            for i in (0..args_len).rev() {
                                let arg = &self.instructions[self.pc + i];
                                match arg {
                                    Instruction::ArgumentName { name } => {
                                        let value = match self.stack.pop() {
                                            Some(value) => value,
                                            None => return Err(VMError::InvalidBytecode),
                                        };
                                        self.symbols.insert(name, value);
                                    }

                                    _ => return Err(VMError::InvalidBytecode),
                                }
                            }
                            
                            self.pc = fn_body_address;
                            
                            for _ in 0..(fn_body_end - fn_body_address) {
                                self.execute_next()?;
                            }

                            self.pc = orig_pc;
                            self.symbols = orig_symbols;

                        } else {
                            return Err(VMError::ErrString(format!("The function `{name}` does not exist!")));
                        }
                    }
                };
            }

            Instruction::Null => self.stack.push(Value::Null),

            Instruction::Delete { name } => {
                // Remove every symbol related to the name
                self.symbols.remove(name);
                self.functions.remove(name);

                if let Ok(..) = get_function(name) {
                    return Err(VMError::ErrString(format!("Cannot delete builtin function `{name}`")));
                }
                self.stack.push(Value::Null);
            }

            Instruction::FunctionDecl { name } => {
                let args = match self.instructions[self.pc] {
                    Instruction::UData { number } => number,
                    _ => return Err(VMError::InvalidBytecode),
                };
                self.pc += 1;

                let end = match self.instructions[self.pc] {
                    Instruction::UData { number } => number,
                    _ => return Err(VMError::InvalidBytecode),
                };
                self.pc += 1;

                let fn_body_address = self.pc + args;
                let fn_body_end = self.pc + end;
                
                self.pc += end;
                self.functions.insert(name, Function::new(args, fn_body_address..fn_body_end));
                self.stack.push(Value::Null);
            }

            Instruction::Print { depth } => {
                let end = self.stack.len();
                let drained = self.stack.drain((end - depth)..(end));
                std::io::stdout().flush().ok();
                for value in drained {
                    print!("{value} ");
                }
                println!();
                self.stack.push(Value::Null);
            }

            instruction => 
                return 
                    Err(
                        VMError::ErrString(
                            format!(
                                "Unexpected instruction {instruction:?} found at {}. This is most probably an error produced by a bug in the bytecode.", 
                                self.pc - 1
                            )
                        )
                    ),
        };
        Ok(())
    }

    pub fn get_symbols(self) -> (HashMap<&'a str, Value>, HashMap<&'a str, Function>) {
        (self.symbols, self.functions)
    }
}