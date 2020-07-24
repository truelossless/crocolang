use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum::*};

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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        for (condition, body) in self.conditions.iter_mut().zip(self.bodies.iter_mut()) {
            let code_pos = &self.code_pos;

            // check if the boolean condition is fullfilled
            let cond_symbol = condition.visit(symtable)?.into_symbol(code_pos)?;

            let cond_ok = cond_symbol
                .borrow()
                .clone()
                .into_primitive()
                .map_err(|_| {
                    CrocoError::new(code_pos, "expected a boolean for the condition".to_owned())
                })?
                .into_bool()
                .map_err(|_| {
                    CrocoError::new(code_pos, "expected a boolean for the condition".to_owned())
                })?;

            // if the condition is fullfilled visit the corresponding body and exit early
            if cond_ok {
                let value = body.visit(symtable)?;
                match value {
                    // propagate the early-return
                    NodeResult::Return(_) | NodeResult::Break | NodeResult::Continue => {
                        return Ok(value)
                    }
                    _ => return Ok(NodeResult::construct_symbol(SymbolContent::Primitive(Void))),
                }
            }
        }

        // if the length doesn't match this means that the last body is an else body
        if self.conditions.len() != self.bodies.len() {
            self.bodies.last_mut().unwrap().visit(symtable)?;
        }

        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(Void)))
    }
}
