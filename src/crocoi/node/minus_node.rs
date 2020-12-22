use crate::error::CrocoError;
use crate::token::LiteralEnum::*;
use crate::{
    ast::node::MinusNode,
    crocoi::{CrocoiNode, INodeResult},
};

#[cfg(feature = "crocoi")]
use crate::crocoi::{utils::get_number_value, ICodegen, ISymbol};

impl CrocoiNode for MinusNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, codegen, &self.code_pos)?
            - get_number_value(&mut self.right, codegen, &self.code_pos)?);
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
