use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{symbol_eq, SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum::*};
/// a node to assign a variable to a certain value
#[derive(Clone)]
pub struct AssignmentNode {
    // variable to assign to (a VarRefNode)
    var: Box<dyn AstNode>,
    // expr assigned
    expr: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl AssignmentNode {
    pub fn new(var: Box<dyn AstNode>, expr: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        AssignmentNode {
            var,
            expr,
            code_pos,
        }
    }
}

impl AstNode for AssignmentNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // get a mutable reference to the variable / field to assign to
        let var = self.var.visit(symtable)?.into_symbol(&self.code_pos)?;

        let expr = self.expr.visit(symtable)?.into_symbol(&self.code_pos)?;
        let expr_borrow = expr.borrow();

        if !symbol_eq(&*var.borrow(), &*expr_borrow) {
            return Err(CrocoError::new(
                &self.code_pos,
                "Cannot change the type of a variable".to_owned(),
            ));
        }

        // clone the contents of the expr
        *var.borrow_mut() = expr_borrow.clone();

        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(Void)))
    }
}
