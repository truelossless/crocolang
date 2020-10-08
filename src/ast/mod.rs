pub mod node;

use crate::error::CrocoError;
use crate::token::CodePos;
use dyn_clonable::*;

#[cfg(feature = "crocoi")]
use crate::crocoi::symbol::{INodeResult, ISymTable};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

#[cfg(feature = "checker")]
use crate::checker::{Checker, CheckerSymbol};

/// a trait used to build node trait objects
#[clonable]
pub trait AstNode: Clone {
    /// the checker running on compiled backends
    #[cfg(feature = "checker")]
    fn check(&mut self, _checker: &mut Checker) -> Result<CheckerSymbol, CrocoError> {
        unimplemented!();
    }

    /// crocoi backend interpreter
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        unimplemented!();
    }

    /// crocol backend code generation, using llvm
    // we could also return a Box<dyn AnyType>, but enum performance should be better
    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        _codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        unimplemented!();
    }

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

#[derive(Clone, Debug)]
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

// this is mostly used by the shunting yard algorithm to provide more info on what we're working with.
pub enum AstNodeType {
    LeafNode,
    UnaryNode,
    BinaryNode,
    NaryNode,
}

/// wether a block node should create a new scope or keep the old one
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
