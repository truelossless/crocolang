use crate::error::CrocoError;
use crate::{
    ast::node::ContinueNode,
    crocoi::{
        symbol::{ICodegen, INodeResult},
        CrocoiNode,
    },
};

impl CrocoiNode for ContinueNode {
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Continue)
    }
}
