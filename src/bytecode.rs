use std::{borrow::Borrow, rc::Rc};
use crate::{ast::{Tree, AST}, errors::Error, instruction::{Instruction, Value}, parser::Parser};

pub struct Bytecode<'a> {
    parser: Parser<'a>,
    error: bool
}

impl<'a> Bytecode<'a> {
    pub fn new(parser: Parser<'a>) -> Self {
        Self { parser, error: false }
    }

    pub fn generate_bytecode(&mut self) -> Vec<Instruction<'a>> {
        let mut complete_bytecode = vec![];
        loop {
            match self.parser.next_expression() {
                Ok(tree) => {
                    complete_bytecode.append(&mut Self::traverse(&tree));
                }
                Err(error) => {
                    self.error = true;
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

            AST::Null => vec![Instruction::Null],

            _ => vec![Instruction::Illegal],
        }
    }
    
}