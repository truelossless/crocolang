use crate::{
    ast::node::VoidNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
    CrocoError,
};

impl CrocolNode for VoidNode {
    fn crocol<'ctx>(
        &mut self,
        _codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        Ok(LNodeResult::Void)
    }
}
