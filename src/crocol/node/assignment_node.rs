use {
    crate::crocol::{LCodegen, LNodeResult},
    inkwell::values::BasicValueEnum,
};

use crate::crocol::CrocolNode;
use crate::{ast::node::AssignmentNode, error::CrocoError};

impl CrocolNode for AssignmentNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let var_ptr = self.var.crocol(codegen)?.into_var(&self.code_pos)?;

        let expr = self
            .expr
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        if expr.symbol_type != var_ptr.symbol_type {
            return Err(CrocoError::type_change_error(&self.code_pos));
        }

        let expr_value: BasicValueEnum = expr.value;

        codegen
            .builder
            .build_store(var_ptr.value.into_pointer_value(), expr_value);
        Ok(LNodeResult::Void)
    }
}
