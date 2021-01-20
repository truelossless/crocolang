use crate::crocol::{LCodegen, LNodeResult};
use crate::error::CrocoError;
use crate::{ast::node::VarCallNode, crocol::CrocolNode};

impl CrocolNode for VarCallNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let symbol = codegen
            .symtable
            .get_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(LNodeResult::Variable(symbol.clone()))
    }
}
