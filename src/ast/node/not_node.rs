use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;
#[derive(Clone)]
/// a node used to invert a boolean value
pub struct NotNode {
    pub bottom: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl NotNode {
    pub fn new(code_pos: CodePos) -> Self {
        NotNode {
            bottom: None,
            code_pos,
        }
    }
}

impl AstNode for NotNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!();
        }
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}

impl BackendNode for NotNode {}
