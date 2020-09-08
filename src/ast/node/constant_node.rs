use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    crocoi::{symbol::SymbolContent, ISymbol},
    token::{CodePos, LiteralEnum},
};

/// a node holding a literal value 
#[derive(Clone)]
pub struct ConstantNode {
    value: LiteralEnum,
    code_pos: CodePos,
}

impl ConstantNode {
    pub fn new(value: LiteralEnum, code_pos: CodePos) -> Self {
        ConstantNode { value, code_pos }
    }
}

impl AstNode for ConstantNode {
    fn visit(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            self.value.clone(),
        )))
    }
}
