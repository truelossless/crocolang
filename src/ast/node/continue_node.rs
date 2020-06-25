use crate::ast::{AstNode, NodeResult};
use crate::symbol::SymTable;
use crate::error::CrocoError;

/// a node representing a continue statement
#[derive(Clone)]
pub struct ContinueNode {}

impl ContinueNode {
    pub fn new() -> Self {
        ContinueNode {}
    }
}

impl AstNode for ContinueNode {
    fn visit(&mut self, _symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        Ok(NodeResult::Continue)
    }
}
