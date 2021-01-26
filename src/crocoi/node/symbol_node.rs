use crate::{ast::AstNode, crocoi::CrocoiNode};
use crate::{ast::BackendNode, error::CrocoError};
use crate::{
    crocoi::{symbol::ISymbol, ICodegen, INodeResult},
    token::CodePos,
};
// A node representing a symbol cannot be backend agnostic: therefore it needs to be created here.
// However, this node is never constructed directly by the parser, but is rather instanciated
// dynamically in crocoi functions.
// TODO: this can probably be removed with a little bit of work.
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

impl AstNode for SymbolNode {}
impl BackendNode for SymbolNode {}

#[cfg(feature = "crocol")]
impl crate::crocol::CrocolNode for SymbolNode {} // required for BackendNode

impl CrocoiNode for SymbolNode {
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(self.value.clone()))
    }
}
