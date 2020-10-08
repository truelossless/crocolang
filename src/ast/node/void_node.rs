use crate::ast::AstNode;
use crate::error::CrocoError;

#[cfg(feature = "crocoi")]
use crate::crocoi::{INodeResult, ISymTable};

/// A node returning a Void NodeResult
#[derive(Clone)]
pub struct VoidNode {}

impl VoidNode {
    pub fn new() -> Self {
        VoidNode {}
    }
}

impl AstNode for VoidNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Void)
    }
}
