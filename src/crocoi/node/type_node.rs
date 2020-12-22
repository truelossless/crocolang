use crate::{
    ast::node::TypeNode,
    crocoi::{CrocoiNode, ICodegen, INodeResult, ISymbol},
    error::CrocoError,
};

impl CrocoiNode for TypeNode {
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(ISymbol::CrocoType(self.value.clone())))
    }
}
