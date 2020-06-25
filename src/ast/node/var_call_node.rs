use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::CodePos;

/// a node holding a variable
#[derive(Clone)]
pub struct VarCallNode {
    name: String,
    code_pos: CodePos,
}

impl VarCallNode {
    pub fn new(name: String, code_pos: CodePos) -> Self {
        VarCallNode { name, code_pos }
    }
}

impl AstNode for VarCallNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let value = symtable
            .get_mut_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        Ok(NodeResult::Symbol(value.clone()))
    }
}
