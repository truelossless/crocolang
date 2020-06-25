use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

/// a node representing an if statement
#[derive(Clone)]
pub struct IfNode {
    // comparison value (a CompareNode)
    left: Option<Box<dyn AstNode>>,
    // if body (a BlockNode)
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl IfNode {
    pub fn new(left: Box<dyn AstNode>, right: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        IfNode {
            left: Some(left),
            right: Some(right),
            code_pos,
        }
    }
}

impl AstNode for IfNode {
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
        // there should always be a boolean condition, check if it's fullfilled
        let cond_ok = self
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
            .into_bool();

        if cond_ok {
            let value = self.right.as_mut().unwrap().visit(symtable)?;
            match value {
                // propagate the early-return
                NodeResult::Return(_) | NodeResult::Break | NodeResult::Continue => {
                    return Ok(value)
                }
                _ => (),
            }
        }

        Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
    }
}
