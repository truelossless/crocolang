use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;
use std::collections::HashMap;

/// a node holding a struct
#[derive(Clone)]
pub struct StructCreateNode {
    pub struct_type: String,
    pub fields: HashMap<String, Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl StructCreateNode {
    pub fn new(
        struct_type: String,
        fields: HashMap<String, Box<dyn BackendNode>>,
        code_pos: CodePos,
    ) -> Self {
        StructCreateNode {
            struct_type,
            code_pos,
            fields,
        }
    }
}

impl AstNode for StructCreateNode {}
impl BackendNode for StructCreateNode {}
