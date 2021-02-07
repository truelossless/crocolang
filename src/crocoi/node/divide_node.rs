use crate::crocoi::{utils::get_value, CrocoiNode, INodeResult};
use crate::error::CrocoError;
use crate::token::LiteralEnum::*;
use crate::{
    ast::node::DivideNode,
    crocoi::{ICodegen, ISymbol},
};

impl CrocoiNode for DivideNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = match (
            get_value(&mut self.left, codegen, &self.code_pos)?,
            get_value(&mut self.right, codegen, &self.code_pos)?,
        ) {
            (Fnum(f1), Fnum(f2)) => Fnum(f1 / f2),
            (Num(n1), Num(n2)) => Num(n1 / n2),
            _ => return Err(CrocoError::divide_error(&self.code_pos)),
        };
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
