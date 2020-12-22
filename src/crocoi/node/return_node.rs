use crate::crocoi::{ICodegen, INodeResult};
use crate::error::CrocoError;
use crate::{ast::node::ReturnNode, crocoi::CrocoiNode};

impl CrocoiNode for ReturnNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        match self.bottom.crocoi(codegen)? {
            INodeResult::Value(val) => Ok(INodeResult::Return(Some(val))),
            INodeResult::Variable(var) => Ok(INodeResult::Return(Some(var.borrow().clone()))),
            INodeResult::Void => Ok(INodeResult::Return(None)),
            _ => Err(CrocoError::invalid_return_value(&self.code_pos)),
        }
    }
}
