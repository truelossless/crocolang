use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::{crocoi::ISymbol, symbol::SymTable};

/// a node representing a continue statement
#[derive(Clone)]
pub struct ContinueNode {}

impl ContinueNode {
    pub fn new() -> Self {
        ContinueNode {}
    }
}

impl AstNode for ContinueNode {
    fn visit(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Continue)
    }
}
