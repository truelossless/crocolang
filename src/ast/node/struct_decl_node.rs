use crate::ast::AstNode;
use crate::ast::BackendNode;
use crate::token::CodePos;
use std::collections::HashMap;

/// A node that contains the declaration of a struct
#[derive(Clone)]
pub struct StructDeclNode {
    /// The name of the struct
    pub name: String,
    /// The body of each method
    pub methods: HashMap<String, Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl StructDeclNode {
    pub fn new(
        name: String,
        methods: HashMap<String, Box<dyn BackendNode>>,
        code_pos: CodePos,
    ) -> Self {
        StructDeclNode {
            name,
            code_pos,
            methods,
        }
    }
}

impl AstNode for StructDeclNode {}
impl BackendNode for StructDeclNode {}
