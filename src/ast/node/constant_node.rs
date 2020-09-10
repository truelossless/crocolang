use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    ast::{AstNode, INodeResult},
    crocol::Codegen,
    crocol::LNodeResult,
};
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
    fn crocoi(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            self.value.clone(),
        )))
    }

    fn crocol<'ctx>(&mut self, codegen: &Codegen<'ctx>) -> Result<LNodeResult<'ctx>, CrocoError> {
        let float_value = self.value.clone().into_num().unwrap();
        let llvm_value = codegen.context.f32_type().const_float(float_value as f64);
        Ok(LNodeResult::Symbol(llvm_value.into()))
    }
}
