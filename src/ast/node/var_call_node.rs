use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{crocoi::ISymbol, token::CodePos};

/// a node holding a variable reference
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
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let symbol = symtable
            .get_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(INodeResult::Symbol(symbol))
    }
}
