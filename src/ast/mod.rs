pub mod node;
mod utils;

use dyn_clone::DynClone;

use crate::symbol::{Symbol, SymTable};
use crate::token::{CodePos};

use crate::error::CrocoError;

// TODO: remove distinctions between left and right and store all node children in a Vec ?
pub trait AstNode: DynClone {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError>;
    fn prepend_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }
    fn add_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }
    fn get_type(&self) -> AstNodeType {
        unimplemented!();
    }
}
dyn_clone::clone_trait_object!(AstNode);

// this is mostly used by the shunting yard algorithm to provide more info on what we're working with.
pub enum AstNodeType {
    LeafNode,
    UnaryNode,
    BinaryNode,
    NaryNode,
}

/// The type of value returned by a node
#[derive(Clone)]
pub enum NodeResult {
    /// a break statement
    Break,
    /// a continue statement
    Continue,
    /// a return statement
    /// e.g return 3
    Return(Symbol),
    /// a symbol
    /// e.g a struct or a primitive
    Symbol(Symbol),
}

impl NodeResult {
    pub fn into_symbol(self, pos: &CodePos) -> Result<Symbol, CrocoError> {
        match self {
            NodeResult::Symbol(s) => Ok(s),
            _ => Err(CrocoError::new(
                pos,
                "Expected a value but got an early-return keyword".to_owned(),
            )),
        }
    }

    pub fn into_return(self) -> Result<Symbol, CrocoError> {
        match self {
            NodeResult::Return(s) => Ok(s),
            _ => panic!("Expected a return value but got an early-return keyword !!"),
        }
    }
}

/// wether a block node should create a nex scope or keep the old one
#[derive(Clone)]
pub enum BlockScope {
    New,
    Keep,
}

impl Default for BlockScope {
    fn default() -> Self {
        BlockScope::New
    }
}