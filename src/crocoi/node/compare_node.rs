use crate::crocoi::CrocoiNode;
use crate::token::{literal_eq, LiteralEnum::*, OperatorEnum};
use crate::{ast::node::CompareNode, error::CrocoError};

use crate::crocoi::{utils::get_value, ICodegen, INodeResult, ISymbol};

impl CrocoiNode for CompareNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, codegen, &self.code_pos)?;
        let right_val = get_value(&mut self.right, codegen, &self.code_pos)?;

        if !literal_eq(&left_val, &right_val) {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot compare different types",
            ));
        }

        if (self.compare_kind != OperatorEnum::Equals
            || self.compare_kind == OperatorEnum::NotEquals)
            && !left_val.is_num()
        {
            return Err(CrocoError::new(&self.code_pos, "can compare only numbers"));
        }

        let value = match self.compare_kind {
            OperatorEnum::Equals => left_val == right_val,
            OperatorEnum::NotEquals => left_val != right_val,
            OperatorEnum::GreaterOrEqual => left_val >= right_val,
            OperatorEnum::GreaterThan => left_val > right_val,
            OperatorEnum::LowerOrEqual => left_val <= right_val,
            OperatorEnum::LowerThan => left_val < right_val,
            _ => unreachable!(),
        };

        Ok(INodeResult::Value(ISymbol::Primitive(Bool(value))))
    }
}
