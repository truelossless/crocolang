use crate::ast::AstNode;
use crate::error::CrocoError;

#[cfg(feature = "crocoi")]
use crate::crocoi::symbol::{INodeResult, ISymTable};

/// a node representing a continue statement
#[derive(Clone)]
pub struct ContinueNode {}

impl ContinueNode {
    pub fn new() -> Self {
        ContinueNode {}
    }
}

impl AstNode for ContinueNode {

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Continue)
    }
}
