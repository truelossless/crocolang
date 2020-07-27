use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{Decl, FunctionDecl, StructDecl, SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum};
use std::collections::HashMap;

/// a node that contains the declaration of a struct
#[derive(Clone)]
pub struct StructDeclNode {
    name: String,
    fields: Option<HashMap<String, SymbolContent>>,
    methods: Option<HashMap<String, FunctionDecl>>,
    code_pos: CodePos,
}

impl StructDeclNode {
    pub fn new(
        name: String,
        fields: HashMap<String, SymbolContent>,
        methods: HashMap<String, FunctionDecl>,
        code_pos: CodePos,
    ) -> Self {
        StructDeclNode {
            name,
            code_pos,
            fields: Some(fields),
            methods: Some(methods),
        }
    }
}

impl AstNode for StructDeclNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // this node is not going to be called again, we can replace
        let struct_decl = StructDecl {
            fields: std::mem::replace(&mut self.fields, None).unwrap(),
            methods: std::mem::replace(&mut self.methods, None).unwrap(),
        };

        symtable
            .register_decl(self.name.clone(), Decl::StructDecl(struct_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(
            LiteralEnum::Void,
        )))
    }
}
