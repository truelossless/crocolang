use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;

/// A node to access symbol fields
#[derive(Clone)]
pub struct DotFieldNode {
    pub field_name: String,
    pub bottom: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl DotFieldNode {
    pub fn new(field_name: String, code_pos: CodePos) -> Self {
        DotFieldNode {
            bottom: None,
            field_name,
            code_pos,
        }
    }
}

impl AstNode for DotFieldNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!()
        }
    }
}

impl BackendNode for DotFieldNode {}
