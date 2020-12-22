use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;

/// a node used for addition and concatenation
#[derive(Clone)]
pub struct PlusNode {
    pub left: Option<Box<dyn BackendNode>>,
    pub right: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl PlusNode {
    pub fn new(code_pos: CodePos) -> Self {
        PlusNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

/// node handling additions and concatenations
impl AstNode for PlusNode {
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

impl BackendNode for PlusNode {}
