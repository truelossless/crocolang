use crate::ast::node::RefNode;
use crate::crocoi::CrocoiNode;
use crate::{crocoi::INodeResult, error::CrocoError};
use {crate::crocoi::ICodegen, std::cell::RefCell, std::rc::Rc};

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
            .into_var_ref(&self.code_pos)
            .map_err(|_| CrocoError::new(&self.code_pos, "dropping a temporary value"))?;

        Ok(INodeResult::Variable(Rc::new(RefCell::new(symbol))))
    }
}
