use std::{borrow::Borrow, collections::HashMap, rc::Rc};
use crate::{ast::{Tree, AST}, errors::Error, instruction::{Instruction, Value}, parser::Parser};

pub struct Bytecode<'a> {
    parser: Parser<'a>,
}

impl<'a> Bytecode<'a> {
    pub fn new(parser: Parser<'a>) -> Self {
        Self { parser }
    }

    pub fn generate_bytecode(&mut self) -> Vec<Instruction<'a>> {
        let mut complete_bytecode = vec![];
        loop {
            match self.parser.next_expression() {
                Ok(tree) => {
                    complete_bytecode.append(&mut Self::traverse(&tree));
                }
                Err(error) => {
                    complete_bytecode.clear();
                    complete_bytecode.push(Instruction::CompileError);    
                    if error != Error::NoResult {
                        println!("{error}");
                    }
                }
            }

            if self.parser.eof {
                break;
            }
        }
        complete_bytecode
    }

    pub fn generate_fn_bytecode(&mut self, old_fn_bytecode: Vec<Instruction<'a>>) -> (Vec<Instruction<'a>>, Vec<Instruction<'a>>) {
        let mut complete_bytecode = old_fn_bytecode.clone();
        let mut function_bytecode = old_fn_bytecode;
        loop {
            match self.parser.next_expression() {
                Ok(tree) => {
                    let mut value = Self::traverse(&tree);
                    if let Instruction::FunctionDecl { .. } = value[0] {
                        function_bytecode.append(&mut (value.clone()))
                    };
                    complete_bytecode.append(&mut value);
                }
                Err(error) => {
                    complete_bytecode.clear();
                    complete_bytecode.push(Instruction::CompileError);
                    function_bytecode.clear();    
                    if error != Error::NoResult {
                        println!("{error}");
                    }
                }
            }

            if self.parser.eof {
                break;
            }
        }

        (complete_bytecode, function_bytecode)
    }


    pub fn get_fn_symbols(self) -> HashMap<&'a str, usize> {
        self.parser.get_fn_symbols()
    }


    fn traverse(tree: &Rc<Tree<'a>>) -> Vec<Instruction<'a>> {
        match tree.ast.borrow() {
            AST::Number { value } => {
                vec![Instruction::Load { value: Value::Number(*value) }]
            },

            AST::BinaryOp { lhs, rhs, op } => {
                let mut instructions = Self::traverse(lhs);
                instructions.append(&mut Self::traverse(rhs));
                instructions.push(Instruction::Binary { operator: *op });
                instructions
            },

            AST::UnaryOp { rhs, op } => {
                let mut instructions = Self::traverse(rhs);
                instructions.push(Instruction::Unary { operator: *op });
                instructions
            },

            AST::Declare { identifier, .. } => {
                vec![Instruction::LoadSymbolName { name: identifier }]
            }

            AST::DeclareAssign { identifier, value, .. } => {
                let mut instructions = Self::traverse(value);
                instructions.push(Instruction::LoadSymbol { name: identifier });
                instructions
            }

            AST::Assign { identifier, value, .. } => {
                let mut instructions = Self::traverse(value);
                instructions.push(Instruction::ReloadSymbol { name: identifier });
                instructions
            }

            AST::AssignOp { identifier, operator, value, .. } => {
                let mut instructions = Self::traverse(value);
                instructions.push(Instruction::ReloadSymbolOp { name: identifier, operator: *operator });
                instructions
            }
            
            AST::Identifier { name } => {
                vec![Instruction::CallSymbol { name }]
            }

            AST::Output { value } => {
                let mut instructions = Self::traverse(value);
                instructions.push(Instruction::Output);
                instructions
            }

            AST::FunctionCall { name, expressions } => {
                let mut instructions = vec![];
                for expr in expressions {
                    instructions.extend(Self::traverse(expr));
                }
                instructions.push(Instruction::FunctionCall { name });
                instructions
            }

            AST::FunctionDecl { name, arguments, body } => {
                let mut instructions = vec![Instruction::FunctionDecl { name, args: arguments.len(), end: 0 }];
                instructions.extend(arguments.iter().map(|name| Instruction::ArgumentName { name }));
                instructions.extend(Self::traverse(body));
                let end = instructions.len() - 1;
                instructions[0] = Instruction::FunctionDecl { name, args: arguments.len(), end };
                instructions
            }

            AST::Delete { name } => vec![Instruction::Delete { name }],

            AST::Null => vec![Instruction::Null],

            _ => vec![Instruction::Illegal],
        }
    }
    
}