use crate::token::CodePos;
use crate::{ast::AstNode, symbol::FunctionDecl};
use crate::{ast::BackendNode, symbol_type::SymbolType};
use std::collections::{BTreeMap, HashMap};

/// A node that contains the declaration of a struct
#[derive(Clone)]
pub struct StructDeclNode {
    // the name of the struct
    pub name: String,
    // the struct's fields
    pub fields: Option<BTreeMap<String, SymbolType>>,
    // for each method, its declaration and its body
    pub methods: Option<HashMap<String, (FunctionDecl, Box<dyn BackendNode>)>>,
    pub code_pos: CodePos,
}

impl StructDeclNode {
    pub fn new(
        name: String,
        fields: BTreeMap<String, SymbolType>,
        methods: HashMap<String, (FunctionDecl, Box<dyn BackendNode>)>,
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

impl AstNode for StructDeclNode {}
impl BackendNode for StructDeclNode {}
