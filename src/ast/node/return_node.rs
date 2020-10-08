use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::crocoi::{INodeResult, ISymTable};

/// A node returning a value from a block
#[derive(Clone)]
pub struct ReturnNode {
    bottom: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl ReturnNode {
    pub fn new(bottom: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        ReturnNode { bottom, code_pos }
    }
}

impl AstNode for ReturnNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        match self.bottom.crocoi(symtable)? {
            INodeResult::Value(val) => Ok(INodeResult::Return(Some(val))),
            INodeResult::Variable(var) => Ok(INodeResult::Return(Some(var.borrow().clone()))),
            INodeResult::Void => Ok(INodeResult::Return(None)),
            _ => Err(CrocoError::new(
                &self.code_pos,
                "expected a valid return value",
            )),
        }
    }
}
