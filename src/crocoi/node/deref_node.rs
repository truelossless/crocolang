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
            .into_symbol(&self.code_pos)?;

        let deref_symbol = match symbol {
            ISymbol::Ref(r) => r,
            _ => return Err(CrocoError::dereference_error(&self.code_pos)),
        };

        Ok(INodeResult::Variable(deref_symbol))
    }
}
