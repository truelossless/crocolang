use crate::crocoi::INodeResult;
use crate::crocoi::{utils::get_value, CrocoiNode};
use crate::{ast::node::UnaryMinusNode, error::CrocoError};

use crate::{
    crocoi::{ICodegen, ISymbol},
    token::LiteralEnum::*,
};

impl CrocoiNode for UnaryMinusNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = match get_value(&mut self.bottom, codegen, &self.code_pos)? {
            Num(n) => Num(-n),
            Fnum(f) => Fnum(-f),
            _ => return Err(CrocoError::unary_minus_error(&self.code_pos)),
        };
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
