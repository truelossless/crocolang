pub mod node;

use dyn_clonable::*;

#[cfg(feature = "crocoi")]
use crate::crocoi::{ISymbol, INodeResult};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

use crate::error::CrocoError;
use crate::symbol::SymTable;

/// a trait used to build node trait objects
#[clonable]
pub trait AstNode: Clone {
    /// crocoi backend interpreter
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        unimplemented!();
    }

    /// crocol backend code generation, using llvm
    // we could also return a Box<dyn AnyType>, but enum performance should be better
    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        _codegen: &Codegen<'ctx>,
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

// this is mostly used by the shunting yard algorithm to provide more info on what we're working with.
pub enum AstNodeType {
    LeafNode,
    UnaryNode,
    BinaryNode,
    NaryNode,
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
