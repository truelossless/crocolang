use crate::crocoi::CrocoiNode;
use crate::{ast::node::VarCallNode, error::CrocoError};

use crate::crocoi::{ICodegen, INodeResult};

impl CrocoiNode for VarCallNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let symbol = codegen
            .symtable
            .get_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(INodeResult::Variable(symbol.clone()))
    }
}
