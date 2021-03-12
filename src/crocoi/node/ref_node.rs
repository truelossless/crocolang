use crate::crocoi::CrocoiNode;
use crate::crocoi::ICodegen;
use crate::{ast::node::RefNode, crocoi::ISymbol};
use crate::{crocoi::INodeResult, error::CrocoError};

impl CrocoiNode for RefNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // it only make sense to create a reference to a reference or a variable, everything else is just
        // dropping temporary values
        // e.g &12, &[3, 4] ...

        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocoi(codegen)?
            .into_var(&self.code_pos)
            .map_err(|_| CrocoError::tmp_value_borrow(&self.code_pos))?;

        Ok(INodeResult::Value(ISymbol::Ref(symbol)))
    }
}
