use crate::error::CrocoError;
use crate::{ast::node::BreakNode, crocoi::CrocoiNode};

use crate::crocoi::{ICodegen, INodeResult};

impl CrocoiNode for BreakNode {
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Break)
    }
}
