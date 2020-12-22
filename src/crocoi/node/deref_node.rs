use crate::{ast::node::DerefNode, crocoi::CrocoiNode};
use crate::{crocoi::INodeResult, error::CrocoError};

use crate::crocoi::symbol::{ICodegen, ISymbol};

impl CrocoiNode for DerefNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocoi(codegen)?
            .into_var(&self.code_pos)?;

        let deref_symbol = match symbol.borrow().clone() {
            ISymbol::Ref(r) => r,
            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "cannot dereference this variable",
                ))
            }
        };

        Ok(INodeResult::Variable(deref_symbol))
    }
}
