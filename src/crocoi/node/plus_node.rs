use crate::ast::node::PlusNode;
use crate::crocoi::{symbol::ISymbol, utils::get_value, CrocoiNode, ICodegen, INodeResult};
use crate::error::CrocoError;
use crate::token::LiteralEnum::*;

impl CrocoiNode for PlusNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, codegen, &self.code_pos)?;
        let right_val = get_value(&mut self.right, codegen, &self.code_pos)?;

        // different kinds of additions can happen (concatenation or number addition)
        let value = match (left_val, right_val) {
            (Num(n1), Num(n2)) => Num(n1 + n2),
            (Fnum(n1), Fnum(n2)) => Fnum(n1 + n2),
            (Str(s1), Str(s2)) => Str(format!("{}{}", s1, s2)),
            _ => return Err(CrocoError::add_error(&self.code_pos)),
        };
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
}
