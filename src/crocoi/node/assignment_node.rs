use crate::{
    ast::node::AssignmentNode,
    crocoi::{symbol::get_symbol_type, CrocoiNode, ICodegen, INodeResult},
};

use crate::error::CrocoError;

impl CrocoiNode for AssignmentNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // get a mutable reference to the variable / field to assign to
        let var = self
            .var
            .crocoi(codegen)?
            .into_var(&self.code_pos)
            .map_err(|_| CrocoError::new(&self.code_pos, "can't assign to this expression"))?;
        let expr = self.expr.crocoi(codegen)?.into_symbol(&self.code_pos)?;

        if get_symbol_type(&*var.borrow()) != get_symbol_type(&expr) {
            return Err(CrocoError::type_change_error(&self.code_pos));
        }

        // assign to the variable the content of the expression
        *var.borrow_mut() = expr;

        Ok(INodeResult::Void)
    }
}
