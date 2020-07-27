use crate::ast::{AstNode, NodeResult};
use crate::symbol::{Decl, FunctionDecl, SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum::*};

use crate::error::CrocoError;

/// function declaration node
#[derive(Clone)]
pub struct FunctionDeclNode {
    name: String,
    fn_decl: Option<FunctionDecl>,
    code_pos: CodePos,
}

impl FunctionDeclNode {
    pub fn new(name: String, fn_decl: FunctionDecl, code_pos: CodePos) -> Self {
        FunctionDeclNode {
            name,
            fn_decl: Some(fn_decl),
            code_pos,
        }
    }
}

impl AstNode for FunctionDeclNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // once the function is declared we can move out its content since this node is not going to be used again
        let fn_decl = std::mem::replace(&mut self.fn_decl, None).unwrap();

        symtable
            .register_decl(self.name.clone(), Decl::FunctionDecl(fn_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(Void)))
    }
}
