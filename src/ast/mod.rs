pub mod node;

use crate::error::CrocoError;
use crate::token::CodePos;
use dyn_clonable::*;

/// A trait used to build node trait objects
#[clonable]
pub trait AstNode: Clone {
    /// Adds a child before the existing children
    fn prepend_child(&mut self, _node: Box<dyn BackendNode>) {
        unimplemented!();
    }

    /// Adds a child after the existing children
    fn add_child(&mut self, _node: Box<dyn BackendNode>) {
        unimplemented!();
    }

    /// Returns the arity of the node
    fn get_type(&self) -> AstNodeType {
        unimplemented!();
    }
}

/// A node implementing all backends
// This adds one layer of indirection (additional pointer in a vtable)
// but it's needed since trait objects don't support multiple traits yet
// relevant issue: https://github.com/rust-lang/rfcs/issues/2035
#[cfg(all(feature = "crocoi", feature = "crocol"))]
#[clonable]
pub trait BackendNode: crate::crocoi::CrocoiNode + crate::crocol::CrocolNode + Clone {}

#[cfg(all(feature = "crocoi", not(feature = "crocol")))]
#[clonable]
pub trait BackendNode: crate::crocoi::CrocoiNode + Clone {}

#[cfg(all(feature = "crocol", not(feature = "crocoi")))]
#[clonable]
pub trait BackendNode: crate::crocol::CrocolNode + Clone {}

#[derive(Clone, Debug)]
/// The result of a node
pub enum NodeResult<T, U> {
    /// a break statement
    Break,
    /// a continue statement
    Continue,
    /// a return statement
    /// e.g return 3
    Return(Option<T>),
    /// a symbol value
    /// e.g a struct or primitive
    Value(T),
    /// a reference to a variable in the symtable
    // for interpreted backends it's going to be refcell'd so we can mutate the value
    // it doesn't need to be for compiled backends
    Variable(U),
    /// when a node returns nothing
    Void,
}

impl<T, U> NodeResult<T, U> {
    pub fn into_value(self, pos: &CodePos) -> Result<T, CrocoError> {
        match self {
            NodeResult::Value(s) => Ok(s),
            NodeResult::Void => panic!("bruh"),
            NodeResult::Variable(_) | NodeResult::Return(_) => panic!(),
            _ => Err(CrocoError::new(
                pos,
                "expected a value but got an early-return keyword",
            )),
        }
    }

    pub fn as_value(&self, pos: &CodePos) -> Result<&T, CrocoError> {
        match self {
            NodeResult::Value(s) => Ok(s),
            _ => Err(CrocoError::new(
                pos,
                "expected a value but got an early-return keyword",
            )),
        }
    }

    pub fn into_var(self, pos: &CodePos) -> Result<U, CrocoError> {
        match self {
            NodeResult::Variable(s) => Ok(s),
            NodeResult::Return(_) | NodeResult::Value(_) => panic!(),
            _ => Err(CrocoError::new(
                pos,
                "expected a value but got an early-return keyword",
            )),
        }
    }

    pub fn into_return(self) -> Result<Option<T>, CrocoError> {
        match self {
            NodeResult::Return(s) => Ok(s),
            NodeResult::Value(_) | NodeResult::Variable(_) => panic!(),
            _ => panic!("Expected a return value but got an early-return keyword"),
        }
    }
}

// This is mostly used by the shunting yard algorithm to provide more info on what we're working with.
pub enum AstNodeType {
    LeafNode,
    UnaryNode,
    BinaryNode,
    NaryNode,
}

/// Wether a block node should create a new scope or keep the old one
#[derive(Clone, PartialEq)]
pub enum BlockScope {
    New,
    Keep,
    Function,
}

impl Default for BlockScope {
    fn default() -> Self {
        BlockScope::New
    }
}
