use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::crocoi::ISymTable;

/// a node representing a while statement
#[derive(Clone)]
pub struct WhileNode {
    // comparison value (a CompareNode)
    left: Option<Box<dyn AstNode>>,
    // while body (a BlockNode)
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl WhileNode {
    pub fn new(left: Box<dyn AstNode>, right: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        WhileNode {
            left: Some(left),
            right: Some(right),
            code_pos,
        }
    }
}

impl AstNode for WhileNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        loop {
            // loop while the condition is ok
            let cond_symbol = self
                .left
                .as_mut()
                .unwrap()
                .crocoi(symtable)?
                .into_value(&self.code_pos)?;

            let condition = cond_symbol
                .into_primitive()
                .map_err(|_| {
                    CrocoError::new(&self.code_pos, "expected a boolean for the condition")
                })?
                .into_bool()
                .map_err(|_| {
                    CrocoError::new(&self.code_pos, "expected a boolean for the condition")
                })?;

            if !condition {
                break;
            }

            let value = self.right.as_mut().unwrap().crocoi(symtable)?;
            match value {
                // propagate the early-return
                INodeResult::Return(_) => return Ok(value),
                INodeResult::Break => return Ok(INodeResult::Void),
                INodeResult::Value(_) | INodeResult::Continue => (),
                _ => unreachable!(),
            }
        }

        Ok(INodeResult::Void)
    }
}
