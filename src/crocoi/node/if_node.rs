use crate::{ast::node::IfNode, crocoi::CrocoiNode};
use crate::{crocoi::INodeResult, error::CrocoError};

use crate::crocoi::ICodegen;

impl CrocoiNode for IfNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let code_pos = &self.code_pos;

        for (condition, body) in self.conditions.iter_mut().zip(self.bodies.iter_mut()) {
            // check if the boolean condition is fullfilled
            let cond_symbol = condition.crocoi(codegen)?.into_symbol(code_pos)?;

            let cond_ok = cond_symbol
                .into_primitive()
                .map_err(|_| CrocoError::condition_not_bool_error(code_pos))?
                .into_bool()
                .map_err(|_| CrocoError::condition_not_bool_error(code_pos))?;

            // if the condition is fullfilled visit the corresponding body and exit early
            if cond_ok {
                let value = body.crocoi(codegen)?;
                match value {
                    // propagate the early-return
                    INodeResult::Return(_) | INodeResult::Break | INodeResult::Continue => {
                        return Ok(value)
                    }
                    _ => {
                        return Ok(INodeResult::Void);
                    }
                }
            }
        }

        // if the length doesn't match this means that the last body is an else body
        if self.conditions.len() != self.bodies.len() {
            self.bodies.last_mut().unwrap().crocoi(codegen)?;
        }

        Ok(INodeResult::Void)
    }
}
