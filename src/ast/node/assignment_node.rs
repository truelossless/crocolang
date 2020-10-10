#[cfg(feature = "checker")]
use crate::checker::{Checker, CheckerSymbol};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::get_symbol_type, INodeResult, ISymTable};

use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::token::CodePos;
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
    #[cfg(feature = "checker")]
    fn check(&mut self, checker: &mut Checker) -> Result<CheckerSymbol, CrocoError> {
        let var = self.var.check(checker)?.into_value(&self.code_pos)?;
        let expr = self.expr.check(checker)?.into_value(&self.code_pos)?;

        if !var.eq(&expr) {
            Err(CrocoError::new(
                &self.code_pos,
                "cannot change the type of a variable",
            ))
        } else {
            Ok(CheckerSymbol::new_unknown_value())
        }
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        // get a mutable reference to the variable / field to assign to
        let var = self
            .var
            .crocoi(symtable)?
            .into_var(&self.code_pos)
            .map_err(|_| CrocoError::new(&self.code_pos, "can't assign to this expression"))?;
        let expr = self.expr.crocoi(symtable)?.into_value(&self.code_pos)?;

        if !get_symbol_type(&*var.borrow()).eq(&get_symbol_type(&expr)) {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot change the type of a variable",
            ));
        }

        // assign to the variable the content of the expression
        *var.borrow_mut() = expr;

        Ok(INodeResult::Void)
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let var_ptr = self.var.crocol(codegen)?.into_var(&self.code_pos)?;
        let expr = self.expr.crocol(codegen)?.into_value(&self.code_pos)?;

        codegen
            .builder
            .build_store(var_ptr.value.into_pointer_value(), expr.value);
        Ok(LNodeResult::Void)
    }
}
