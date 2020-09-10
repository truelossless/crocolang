use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{crocoi::ISymbol, token::CodePos};

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
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Return(
            self.bottom.crocoi(symtable)?.into_symbol(&self.code_pos)?,
        ))
    }
}
