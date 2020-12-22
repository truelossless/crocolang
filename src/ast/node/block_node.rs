use crate::ast::{AstNode, BackendNode, BlockScope};

/// A node containing multiple instructions
/// creates a new scope, or not
/// e.g: if body, function body, etc.
#[derive(Clone)]
pub struct BlockNode {
    // all instructions of the block node
    pub body: Vec<Box<dyn BackendNode>>,
    pub scope: BlockScope,
}

impl BlockNode {
    pub fn new(scope: BlockScope) -> Self {
        BlockNode {
            body: Vec::new(),
            scope,
        }
    }
}

impl AstNode for BlockNode {
    fn prepend_child(&mut self, node: Box<dyn BackendNode>) {
        self.body.insert(0, node);
    }

    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        self.body.push(node);
    }
}

impl BackendNode for BlockNode {}
