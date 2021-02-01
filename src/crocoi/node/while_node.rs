use crate::crocoi::ICodegen;
use crate::{ast::node::WhileNode, crocoi::CrocoiNode};
use crate::{crocoi::INodeResult, error::CrocoError};

impl CrocoiNode for WhileNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        loop {
            // loop while the condition is ok
            let cond_symbol = self
                .left
                .as_mut()
                .unwrap()
                .crocoi(codegen)?
                .into_value(&self.code_pos)?;

            let condition = cond_symbol
                .into_primitive()
                .map_err(|_| CrocoError::condition_not_bool_error(&self.code_pos))?
                .into_bool()
                .map_err(|_| CrocoError::condition_not_bool_error(&self.code_pos))?;

            if !condition {
                break;
            }

            let value = self.right.as_mut().unwrap().crocoi(codegen)?;
            match value {
                // propagate the early-return
                INodeResult::Return(_) => return Ok(value),
                INodeResult::Break => return Ok(INodeResult::Void),
                _ => (),
            }
        }

        Ok(INodeResult::Void)
    }
}
