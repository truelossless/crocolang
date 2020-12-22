use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;
/// a node representing a while statement
#[derive(Clone)]
pub struct WhileNode {
    // comparison value (a CompareNode)
    pub left: Option<Box<dyn BackendNode>>,
    // while body (a BlockNode)
    pub right: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl WhileNode {
    pub fn new(left: Box<dyn BackendNode>, right: Box<dyn BackendNode>, code_pos: CodePos) -> Self {
        WhileNode {
            left: Some(left),
            right: Some(right),
            code_pos,
        }
    }
}

impl AstNode for WhileNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }
}

impl BackendNode for WhileNode {}
