use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;

/// a node representing a break statement
#[derive(Clone)]
pub struct BreakNode {}

impl BreakNode {
    pub fn new() -> Self {
        BreakNode {}
    }
}

impl AstNode for BreakNode {
    fn visit(&mut self, _symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        Ok(NodeResult::Break)
    }
}