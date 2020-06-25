use crate::ast::{AstNode, BlockScope, NodeResult};
use crate::symbol::{SymTable, Symbol};
use crate::token::{LiteralEnum::*};
use crate::error::CrocoError;

/// node containing multiple instructions
/// creates a new scope, or not
/// e.g: if body, function body, etc.
#[derive(Clone)]
pub struct BlockNode {
    // all instructions of the block node
    body: Vec<Box<dyn AstNode>>,
    scope: BlockScope,
    // instructions that get prepended, e.g variables in fn calls
    // prepended: Vec<Box<dyn AstNode>>,
    // same as previous, useful for future defer calls
    // appended: Vec<Box<dyn AstNode>>
}

impl BlockNode {
    pub fn new(scope: BlockScope) -> Self {
        BlockNode {
            body: Vec::new(),
            scope
            // prepended: Vec::new(),
            // appended: Vec::new(),
        }
    }
}

impl AstNode for BlockNode {
    fn prepend_child(&mut self, node: Box<dyn AstNode>) {
        self.body.insert(0, node);
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        self.body.push(node);
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // push a new scope if needed
        match self.scope {
            BlockScope::New => symtable.add_scope(),
            BlockScope::Keep => (),
        }

        // early return from the block
        let mut value = NodeResult::Symbol(Symbol::Primitive(Void));
        // iterate over all nodes in the body
        for node in self.body.iter_mut()
        // .chain(self.prepended.iter_mut())
        // .chain(self.appended.iter_mut())
        {
            value = node.visit(symtable)?;

            match value {
                // propagate the early-returns until something catches it
                NodeResult::Return(_) | NodeResult::Break | NodeResult::Continue => break,
                _ => (),
            }
        }

        // clean up the injected statements
        // self.prepended.clear();
        // self.appended.clear();

        // return void if there is no return value
        if let NodeResult::Symbol(_) = value {
            value = NodeResult::Symbol(Symbol::Primitive(Void))
        }

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New => symtable.drop_scope(),
            BlockScope::Keep => (),
        }

        Ok(value)
    }
}