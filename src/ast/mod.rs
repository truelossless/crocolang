pub mod node;

use dyn_clonable::*;

use crate::crocoi::{INodeResult, ISymbol};
use crate::crocol::{Codegen, LNodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;

/// a trait used to build node trait objects
#[clonable]
pub trait AstNode: Clone {
    /// recursively visit the node and its children and returns its value
    // TODO: proper feature separation by moving the crocoi & crocol implementation to their respective folders.
    fn crocoi(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        unimplemented!();
    }

    // we could also return a Box<dyn AnyType>, but enum performance should be better
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
