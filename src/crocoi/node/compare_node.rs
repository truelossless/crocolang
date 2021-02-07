use crate::token::{LiteralEnum::*, OperatorEnum};
use crate::{ast::node::CompareNode, error::CrocoError};
use crate::{crocoi::CrocoiNode, token::literal_eq};

use crate::crocoi::{utils::get_value, ICodegen, INodeResult, ISymbol};

impl CrocoiNode for CompareNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, codegen, &self.code_pos)?;
        let right_val = get_value(&mut self.right, codegen, &self.code_pos)?;

        // make sure we can compare our values
        // that is, if they are both a (f)num, or if they are of the same type
        match (&left_val, &right_val) {
            _ if literal_eq(&left_val, &right_val) => (),
            (Fnum(_), Num(_)) | (Num(_), Fnum(_)) => (),
            _ => return Err(CrocoError::compare_different_types_error(&self.code_pos)),
        }

        if (self.compare_kind != OperatorEnum::Equals
            && self.compare_kind != OperatorEnum::NotEquals)
            && !left_val.is_num_fnum()
        {
            return Err(CrocoError::compare_numbers_only_error(&self.code_pos));
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
