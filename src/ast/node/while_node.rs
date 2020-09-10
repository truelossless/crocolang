use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{crocoi::{symbol::SymbolContent, ISymbol}, token::{CodePos, LiteralEnum::*}};

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
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        loop {
            // loop while the condition is ok
            let cond_symbol = self
                .left
                .as_mut()
                .unwrap()
                .crocoi(symtable)?
                .into_symbol(&self.code_pos)?;

            let condition = cond_symbol
                .borrow()
                .clone()
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
                INodeResult::Break => {
                    return Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
                        Void,
                    )))
                }
                INodeResult::Symbol(_) | INodeResult::Continue => (),
            }
        }

        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            Void,
        )))
    }
}
