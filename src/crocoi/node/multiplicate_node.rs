use crate::{
    ast::node::MultiplicateNode,
    crocoi::{utils::get_number_value, CrocoiNode, ICodegen, INodeResult, ISymbol},
};

use crate::error::CrocoError;
use crate::token::LiteralEnum::*;

impl CrocoiNode for MultiplicateNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, codegen, &self.code_pos)?
            * get_number_value(&mut self.right, codegen, &self.code_pos)?);
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
