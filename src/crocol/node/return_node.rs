use crate::error::CrocoError;
use crate::{
    ast::node::ReturnNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
};

impl CrocolNode for ReturnNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        match self.bottom.crocol(codegen)? {
            LNodeResult::Value(val) => Ok(LNodeResult::Return(Some(val))),
            LNodeResult::Variable(var) => Ok(LNodeResult::Return(Some(var))),
            LNodeResult::Void => Ok(LNodeResult::Return(None)),
            _ => Err(CrocoError::invalid_return_value(&self.code_pos)),
        }
    }
}
