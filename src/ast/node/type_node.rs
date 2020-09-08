use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    crocoi::{symbol::SymbolContent, ISymbol},
    symbol_type::SymbolType,
    token::CodePos,
};

/// a node holding a type value
#[derive(Clone)]
pub struct TypeNode {
    value: SymbolType,
    code_pos: CodePos,
}

impl TypeNode {
    pub fn new(value: SymbolType, code_pos: CodePos) -> Self {
        TypeNode { value, code_pos }
    }
}

impl AstNode for TypeNode {
    fn visit(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::construct_symbol(SymbolContent::CrocoType(
            self.value.clone(),
        )))
    }
}
