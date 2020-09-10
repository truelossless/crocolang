use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{crocoi::{ISymbol, symbol::SymbolContent, INodeResult}, token::CodePos};

// A node representing a symbol cannot be backend agnostic: therefore it needs to be created here.
// However, this node is never constructed directly by the parser, but is rather instanciated
// dynamically in functions.
/// a node holding a symbol
#[derive(Clone)]
pub struct SymbolNode {
    value: SymbolContent,
    code_pos: CodePos,
}

impl SymbolNode {
    pub fn new(value: SymbolContent, code_pos: CodePos) -> Self {
        SymbolNode { value, code_pos }
    }
}

impl AstNode for SymbolNode {
    fn crocoi(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::construct_symbol(self.value.clone()))
    }
}
