use crate::crocoi::symbol::Function;
use crate::{
    ast::node::FunctionDeclNode,
    crocoi::{CrocoiNode, INodeResult},
};

use crate::crocoi::ICodegen;
use crate::error::CrocoError;

impl CrocoiNode for FunctionDeclNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        codegen.functions.insert(
            self.name.clone(),
            Function::Regular(self.fn_body.take().unwrap()),
        );

        Ok(INodeResult::Void)
    }
}
