use crate::error::CrocoError;
use crate::{
    ast::node::ConstantNode,
    crocoi::{CrocoiNode, ICodegen, INodeResult, ISymbol},
};

impl CrocoiNode for ConstantNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(ISymbol::Primitive(self.value.clone())))
    }
}
