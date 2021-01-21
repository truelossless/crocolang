use crate::ast::node::StructDeclNode;
use crate::crocoi;
use crate::crocoi::{ICodegen, INodeResult};
use crate::{crocoi::CrocoiNode, error::CrocoError};

impl CrocoiNode for StructDeclNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        for (method_name, method_body) in self.methods.drain() {
            // register the function body
            codegen
                .functions
                .insert(method_name, crocoi::symbol::Function::Regular(method_body));
        }

        Ok(INodeResult::Void)
    }
}
