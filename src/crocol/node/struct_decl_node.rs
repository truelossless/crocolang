use crate::{
    ast::node::StructDeclNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
    CrocoError,
};

impl CrocolNode for StructDeclNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        for (method_name, method_body) in self.methods.drain() {
            codegen.build_function(&method_name, method_body, &self.code_pos)?;
        }

        Ok(LNodeResult::Void)
    }
}
