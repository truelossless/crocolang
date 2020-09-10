use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::{Decl, FunctionDecl, StructDecl, SymTable};
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, LiteralEnum}, crocoi::{symbol::SymbolContent, ISymbol},
};
use std::collections::{BTreeMap, HashMap};

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
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        // this node is not going to be called again, we can replace

        let struct_decl = StructDecl {
            fields: std::mem::replace(&mut self.fields, None).unwrap(),
            methods: std::mem::replace(&mut self.methods, None).unwrap(),
        };

        symtable
            .register_decl(self.name.clone(), Decl::StructDecl(struct_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, &e))?;

        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            LiteralEnum::Void,
        )))
    }
}
