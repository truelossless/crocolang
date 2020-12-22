use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;
/// a node to import code from another module, at runtime.
#[derive(Clone)]
pub struct ImportNode {
    pub name: String,
    pub bottom: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl ImportNode {
    pub fn new(name: String, code_pos: CodePos) -> Self {
        ImportNode {
            name,
            bottom: None,
            code_pos,
        }
    }
}

impl AstNode for ImportNode {}
impl BackendNode for ImportNode {}
