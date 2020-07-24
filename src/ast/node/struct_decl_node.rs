use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{Decl, SymTable, Symbol, SymbolContent};
use crate::token::{CodePos, LiteralEnum};
use std::collections::HashMap;

/// a node that contains the declaration of a struct
#[derive(Clone)]
pub struct StructDeclNode {
    name: String,
    fields: Option<HashMap<String, Symbol>>,
    code_pos: CodePos,
}

impl StructDeclNode {
    pub fn new(name: String, fields: HashMap<String, Symbol>, code_pos: CodePos) -> Self {
        StructDeclNode {
            name,
            code_pos,
            fields: Some(fields),
        }
    }
}

impl AstNode for StructDeclNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        symtable
            .register_decl(
                self.name.clone(),
                Decl::StructDecl(std::mem::replace(&mut self.fields, None).unwrap()),
            )
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "a variable with the same name exists in this scope".to_owned(),
                )
            })?;

        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(
            LiteralEnum::Void,
        )))
    }
}
