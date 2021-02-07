use crate::crocoi::{utils::get_value, CrocoiNode};
use crate::error::CrocoError;
use crate::token::LiteralEnum::*;
use crate::{
    ast::node::PowerNode,
    crocoi::{ICodegen, INodeResult, ISymbol},
};

impl CrocoiNode for PowerNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = match (
            get_value(&mut self.left, codegen, &self.code_pos)?,
            get_value(&mut self.right, codegen, &self.code_pos)?,
        ) {
            (Fnum(f1), Fnum(f2)) => Fnum(f1.powf(f2)),
            // TODO: handle panic ?
            (Num(n1), Num(n2)) => Num(n1.pow(n2 as u32)),
            _ => return Err(CrocoError::power_error(&self.code_pos)),
        };
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
