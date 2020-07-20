use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // loop while the condition is ok
        while self
            .left
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)?
            .into_primitive()
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "expected a boolean for the condition".to_owned(),
                )
            })?
            .into_bool()
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "expected a boolean for the condition".to_owned(),
                )
            })?
        {
            let value = self.right.as_mut().unwrap().visit(symtable)?;
            match value {
                // propagate the early-return
                NodeResult::Return(_) => return Ok(value),
                NodeResult::Break => return Ok(NodeResult::Symbol(Symbol::Primitive(Void))),
                NodeResult::Symbol(_) | NodeResult::Continue => (),
            }
        }

        Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
    }
}
