use crate::crocoi::{ICodegen, INodeResult};
use crate::error::CrocoError;
use crate::{ast::node::VoidNode, crocoi::CrocoiNode};

impl CrocoiNode for VoidNode {
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Void)
    }
}
