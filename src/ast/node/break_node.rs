use crate::ast::{AstNode,INodeResult};
use crate::error::CrocoError;
use crate::{crocoi::ISymbol, symbol::SymTable};

/// a node representing a break statement
#[derive(Clone)]
pub struct BreakNode {}

impl BreakNode {
    pub fn new() -> Self {
        BreakNode {}
    }
}

impl AstNode for BreakNode {
    fn visit(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Break)
    }
}
