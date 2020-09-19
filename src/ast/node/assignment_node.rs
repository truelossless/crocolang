#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::SymbolContent, INodeResult, ISymbol};

use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::symbol::{get_symbol_type, SymTable};
use crate::{
    symbol_type::type_eq,
    token::{CodePos, LiteralEnum::*},
};
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
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        // get a mutable reference to the variable / field to assign to
        let var = self.var.crocoi(symtable)?.into_symbol(&self.code_pos)?;

        let expr = self.expr.crocoi(symtable)?.into_symbol(&self.code_pos)?;
        let expr_borrow = &*expr.borrow();

        if !type_eq(
            &get_symbol_type(&*var.borrow()),
            &get_symbol_type(expr_borrow),
        ) {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot change the type of a variable",
            ));
        }

        // clone the contents of the expr
        *var.borrow_mut() = expr_borrow.clone();

        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            Void,
        )))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(&mut self, codegen: &Codegen<'ctx>) -> Result<LNodeResult<'ctx>, CrocoError> {
        let var_ptr = self.var.crocol(codegen)?.into_symbol();

        let expr = self
            .expr
            .crocol(codegen)?
            .into_symbol();
            
        let expr_value = codegen.auto_deref(expr);

        codegen
            .builder
            .build_store(var_ptr.into_pointer_value(), expr_value);

        Ok(LNodeResult::Void)
    }
}
