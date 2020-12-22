use crate::crocoi::CrocoiNode;
use crate::crocoi::INodeResult;
use crate::{ast::node::UnaryMinusNode, error::CrocoError};

use crate::{
    crocoi::{utils::get_number_value, ICodegen, ISymbol},
    token::LiteralEnum::*,
};

impl CrocoiNode for UnaryMinusNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = Num(-get_number_value(
            &mut self.bottom,
            codegen,
            &self.code_pos,
        )?);
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
