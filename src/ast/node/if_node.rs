use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::crocoi::ISymTable;

/// a node representing an if / elif / else structure
#[derive(Clone)]
pub struct IfNode {
    // comparison value (a CompareNode)
    conditions: Vec<Box<dyn AstNode>>,
    // if / elif / else bodies (a BlockNode)
    bodies: Vec<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl IfNode {
    pub fn new(
        conditions: Vec<Box<dyn AstNode>>,
        bodies: Vec<Box<dyn AstNode>>,
        code_pos: CodePos,
    ) -> Self {
        IfNode {
            conditions,
            bodies,
            code_pos,
        }
    }
}

impl AstNode for IfNode {

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        for (condition, body) in self.conditions.iter_mut().zip(self.bodies.iter_mut()) {
            let code_pos = &self.code_pos;

            // check if the boolean condition is fullfilled
            let cond_symbol = condition.crocoi(symtable)?.into_symbol(code_pos)?;

            let cond_ok = cond_symbol
                .into_primitive()
                .map_err(|_| CrocoError::new(code_pos, "expected a boolean for the condition"))?
                .into_bool()
                .map_err(|_| CrocoError::new(code_pos, "expected a boolean for the condition"))?;

            // if the condition is fullfilled visit the corresponding body and exit early
            if cond_ok {
                let value = body.crocoi(symtable)?;
                match value {
                    // propagate the early-return
                    INodeResult::Return(_) | INodeResult::Break | INodeResult::Continue => {
                        return Ok(value)
                    }
                    _ => return Ok(INodeResult::Void),
                }
            }
        }

        // if the length doesn't match this means that the last body is an else body
        if self.conditions.len() != self.bodies.len() {
            self.bodies.last_mut().unwrap().crocoi(symtable)?;
        }

        Ok(INodeResult::Void)
    }
}
