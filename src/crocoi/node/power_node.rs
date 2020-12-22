use crate::crocoi::CrocoiNode;
use crate::error::CrocoError;
use crate::token::LiteralEnum::*;
use crate::{
    ast::node::PowerNode,
    crocoi::{utils::get_number_value, ICodegen, INodeResult, ISymbol},
};

impl CrocoiNode for PowerNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, codegen, &self.code_pos)?
            .powf(get_number_value(&mut self.right, codegen, &self.code_pos)?));
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
