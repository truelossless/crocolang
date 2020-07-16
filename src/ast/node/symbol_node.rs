use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol};
use crate::token::CodePos;

/// a node holding a symbol
#[derive(Clone)]
pub struct SymbolNode {
    value: Symbol,
    code_pos: CodePos,
}

impl SymbolNode {
    pub fn new(value: Symbol, code_pos: CodePos) -> Self {
        SymbolNode { value, code_pos }
    }
}

impl AstNode for SymbolNode {
    fn visit(&mut self, _symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        Ok(NodeResult::Symbol(self.value.clone()))
    }
}
