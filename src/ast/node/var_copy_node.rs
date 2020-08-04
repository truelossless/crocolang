use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::CodePos;

/// a node returning a copy of a variable value
#[derive(Clone)]
pub struct VarCopyNode {
    name: String,
    code_pos: CodePos,
}

impl VarCopyNode {
    pub fn new(name: String, code_pos: CodePos) -> Self {
        VarCopyNode { name, code_pos }
    }
}

impl AstNode for VarCopyNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // TODO: deep clone: here we're only cloning the base symbol but all attributes still points to the same variable
        let value = symtable
            .get_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .borrow()
            .clone();

        Ok(NodeResult::construct_symbol(value))
    }
}
