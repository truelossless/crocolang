use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

/// a node to assign a variable to a certain value
#[derive(Clone)]
pub struct AssignmentNode {
    // variable to assign to
    left: String,
    // expr assigned
    right: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl AssignmentNode {
    pub fn new(var_name: String, expr: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        AssignmentNode {
            left: var_name,
            right: expr,
            code_pos,
        }
    }
}

impl AstNode for AssignmentNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let right_val = self.right.visit(symtable)?.into_symbol(&self.code_pos)?;

        symtable
            .modify_symbol(&self.left, right_val)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
    }
}
