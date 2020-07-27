pub mod node;
mod utils;

use dyn_clone::DynClone;
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol, SymbolContent};
use crate::token::CodePos;

// TODO: remove distinctions between left and right and store all node children in a Vec ?
/// a trait used to build node trait objects
pub trait AstNode: DynClone {
    /// recursively visit the node and its children and returns its value
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError>;

    /// add a child before the existing children
    fn prepend_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }

    /// add a child after the existing children
    fn add_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }

    /// returns the arity of the node
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
    /// convenience function to build a Symbol
    pub fn construct_symbol(symbol_content: SymbolContent) -> NodeResult {
        NodeResult::Symbol(Rc::new(RefCell::new(symbol_content)))
    }

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
