use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;

#[derive(Clone)]
pub struct PowerNode {
    pub left: Option<Box<dyn BackendNode>>,
    pub right: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl PowerNode {
    pub fn new(code_pos: CodePos) -> Self {
        PowerNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

impl AstNode for PowerNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}

impl BackendNode for PowerNode {}
