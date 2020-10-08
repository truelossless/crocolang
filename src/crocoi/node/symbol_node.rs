use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::{
    crocoi::{symbol::ISymbol, INodeResult, ISymTable},
    token::CodePos,
};

// A node representing a symbol cannot be backend agnostic: therefore it needs to be created here.
// However, this node is never constructed directly by the parser, but is rather instanciated
// dynamically in crocoi functions.
/// a node holding a symbol
#[derive(Clone)]
pub struct SymbolNode {
    value: ISymbol,
    code_pos: CodePos,
}

impl SymbolNode {
    pub fn new(value: ISymbol, code_pos: CodePos) -> Self {
        SymbolNode { value, code_pos }
    }
}

impl AstNode for SymbolNode {
    fn crocoi(
        &mut self,
        _symtable: &mut ISymTable,
    ) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(self.value.clone()))
    }
}
