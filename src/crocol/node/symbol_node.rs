use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::{
    crocol::{Codegen, LNodeResult},
    token::CodePos,
};
use inkwell::values::AnyValueEnum;

// A node representing a symbol cannot be backend agnostic: therefore it needs to be created here.
// However, this node is never constructed directly by the parser, but is rather instanciated
// dynamically in functions.
/// a node holding a symbol
#[derive(Clone)]
pub struct SymbolNode<'ctx> {
    value: AnyValueEnum<'ctx>,
    code_pos: CodePos,
}

impl<'ctx> SymbolNode<'ctx> {
    pub fn new(value: AnyValueEnum<'ctx>, code_pos: CodePos) -> Self {
        SymbolNode { value, code_pos }
    }
}

impl AstNode for SymbolNode<'ctx> {
    fn crocol(
        &mut self,
        codegen: &'ctx mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        Ok(LNodeResult::Symbol(self.value))
    }
}
