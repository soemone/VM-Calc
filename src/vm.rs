use std::collections::HashMap;

use crate::{ast::Operator, functions::get_function, instruction::{Instruction, Value}};

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
    pub(crate) symbols: HashMap<&'a str, Value>,
    pub(crate) function_symbols: HashMap<&'a str, (usize, usize, usize)>
}

impl<'a> VM<'a> {
    pub fn new(instructions: Vec<Instruction<'a>>) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            outputs: vec![],
            symbols: HashMap::new(),
            function_symbols: HashMap::new(),
            instructions,
        }
    }

    pub fn new_with_symbols(instructions: Vec<Instruction<'a>>, symbols: HashMap<&'a str, Value>, function_symbols: HashMap<&'a str, (usize, usize, usize)>) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            outputs: vec![],
            symbols,
            function_symbols,
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
            Instruction::Load { value } => self.stack.push(value.clone()),

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
                    _ => unimplemented!()
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

            Instruction::ReloadSymbolOp { name, operator } => {
                match self.symbols.get_mut(name) {
                    Some(value) => {
                        let new_value = match self.stack.pop() {
                            Some(res) => match res {
                                Value::Number(number) => number,
                                Value::Null => return Err(VMError::ErrString(format!("Cannot operate `{name}` on Null type!"))),    
                            },
                            None => return Err(VMError::InvalidBytecode), 
                        };
                        let mut new_number = match value {
                            Value::Number(number) => *number,
                            Value::Null => return Err(VMError::ErrString(format!("Cannot operate Null type `{name}` on expression!"))),
                        };

                        match operator {
                            Operator::PlusEqual => new_number += new_value,
                            Operator::MinusEqual => new_number -= new_value,
                            Operator::DivideEqual => new_number /= new_value,
                            Operator::MultiplyEqual => new_number *= new_value,
                            Operator::ExponentEqual => new_number = f64::powf(new_number, new_value),
                            Operator::BitOrEqual => new_number = (new_number as usize | new_value as usize) as f64,
                            Operator::BitAndEqual => new_number = (new_number as usize & new_value as usize) as f64,
                            Operator::BitXorEqual => new_number = (new_number as usize ^ new_value as usize) as f64,
                            Operator::BitLeftShiftEqual => new_number = ((new_number as usize) << new_value as usize) as f64,
                            Operator::BitRightShiftEqual => new_number = (new_number as usize >> new_value as usize) as f64,
                            _ => unimplemented!()
                        }

                        *value = Value::Number(new_number);  
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
                        if let Some((fn_args_address, fn_body_address, fn_body_end)) = self.function_symbols.get(name) {
                            let orig_pc = self.pc;
                            let orig_symbols = self.symbols.clone();
                            self.pc = *fn_args_address;
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
                            self.pc = *fn_body_address;
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
                if let Some(..) = self.symbols.get(name) { self.symbols.remove(name); }
                else if let Ok(..) = get_function(name) {
                    return Err(VMError::ErrString(format!("Cannot delete builtin function `{name}`")));
                } else if let Some(..) = self.function_symbols.get(name) { self.function_symbols.remove(name); };
                self.stack.push(Value::Null);
            }

            Instruction::FunctionDecl { name, args, end } => {
                let fn_body_address = self.pc + args;
                let fn_args_address = self.pc;
                let fn_body_end = self.pc + end;
                self.function_symbols.insert(name, (fn_args_address, fn_body_address, fn_body_end));
                self.pc += end;
                self.stack.push(Value::Null);
            }

            _ => unimplemented!(),
        };
        Ok(())
    }

    pub fn get_symbols(self) -> (HashMap<&'a str, Value>, HashMap<&'a str, (usize, usize, usize)>) {
        (self.symbols, self.function_symbols)
    }
}