use crate::{
    ast::node::BreakNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
    CrocoError,
};

impl CrocolNode for BreakNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        codegen
            .builder
            .build_unconditional_branch(codegen.current_loop_end_block.unwrap());
        Ok(LNodeResult::Break)
    }
}
