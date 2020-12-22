use crate::{
    ast::node::FunctionDeclNode,
    crocoi::{CrocoiNode, INodeResult},
};
use crate::{crocoi::symbol::Function, symbol::Decl};

use crate::crocoi::ICodegen;
use crate::error::CrocoError;

impl CrocoiNode for FunctionDeclNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // once the function is declared we can move out its content since this node is not going to be used again
        let fn_decl = self.fn_decl.take().unwrap();

        codegen
            .symtable
            .register_decl(self.name.clone(), Decl::FunctionDecl(fn_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, &e))?;

        codegen.functions.insert(
            self.name.clone(),
            Function::Regular(self.fn_body.take().unwrap()),
        );

        Ok(INodeResult::Void)
    }
}
