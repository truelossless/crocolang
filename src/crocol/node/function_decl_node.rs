use crate::crocol::{LCodegen, LNodeResult};
use crate::error::CrocoError;
use crate::{ast::node::FunctionDeclNode, crocol::CrocolNode};

impl CrocolNode for FunctionDeclNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        codegen.build_function(&self.name, self.fn_body.take().unwrap(), &self.code_pos)?;
        Ok(LNodeResult::Void)
    }
}
