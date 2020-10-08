use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::symbol::{Decl, FunctionDecl, StructDecl};
use crate::token::CodePos;
use std::collections::{BTreeMap, HashMap};

#[cfg(feature = "crocoi")]
use crate::{
    crocoi::{INodeResult, ISymTable},
    symbol_type::SymbolType,
};

/// a node that contains the declaration of a struct
#[derive(Clone)]
pub struct StructDeclNode {
    name: String,
    fields: Option<BTreeMap<String, SymbolType>>,
    methods: Option<HashMap<String, FunctionDecl>>,
    code_pos: CodePos,
}

impl StructDeclNode {
    pub fn new(
        name: String,
        fields: BTreeMap<String, SymbolType>,
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
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        // this node is not going to be called again, we can replace

        let struct_decl = StructDecl {
            fields: std::mem::replace(&mut self.fields, None).unwrap(),
            methods: std::mem::replace(&mut self.methods, None).unwrap(),
        };

        symtable
            .register_decl(self.name.clone(), Decl::StructDecl(struct_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, &e))?;

        Ok(INodeResult::Void)
    }
}
