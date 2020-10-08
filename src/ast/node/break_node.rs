use crate::ast::AstNode;
use crate::error::CrocoError;

#[cfg(feature = "crocoi")]
use crate::crocoi::{ISymTable, INodeResult};

/// a node representing a break statement
#[derive(Clone)]
pub struct BreakNode {}

impl BreakNode {
    pub fn new() -> Self {
        BreakNode {}
    }
}

impl AstNode for BreakNode {
    fn crocoi(
        &mut self,
        _symtable: &mut ISymTable,
    ) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Break)
    }
}
