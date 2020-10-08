use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::{
    crocoi::{ISymTable, ISymbol},
    symbol_type::SymbolType,
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

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(ISymbol::CrocoType(self.value.clone())))
    }
}
