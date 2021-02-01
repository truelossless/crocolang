use crate::{
    ast::node::ContinueNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
    CrocoError,
};

impl CrocolNode for ContinueNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        codegen
            .builder
            .build_unconditional_branch(codegen.current_loop_block.unwrap());
        Ok(LNodeResult::Continue)
    }
}
