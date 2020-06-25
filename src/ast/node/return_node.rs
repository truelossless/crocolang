use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::CodePos;

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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        Ok(NodeResult::Return(
            self.bottom.visit(symtable)?.into_symbol(&self.code_pos)?,
        ))
    }
}
