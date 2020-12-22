use crate::{
    ast::{AstNode, BackendNode},
    token::CodePos,
};

/// a node holding a variable reference
#[derive(Clone)]
pub struct VarCallNode {
    pub name: String,
    pub code_pos: CodePos,
}

impl VarCallNode {
    pub fn new(name: String, code_pos: CodePos) -> Self {
        VarCallNode { name, code_pos }
    }
}

impl AstNode for VarCallNode {}
impl BackendNode for VarCallNode {}
