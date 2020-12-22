use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;

#[derive(Clone)]

/// A node used to access an array element at a certain index.
pub struct ArrayIndexNode {
    pub array: Option<Box<dyn BackendNode>>,
    pub index: Box<dyn BackendNode>,
    pub code_pos: CodePos,
}

impl ArrayIndexNode {
    pub fn new(index: Box<dyn BackendNode>, code_pos: CodePos) -> Self {
        ArrayIndexNode {
            array: None,
            index,
            code_pos,
        }
    }
}

impl AstNode for ArrayIndexNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.array.is_none() {
            self.array = Some(node);
        } else {
            unreachable!()
        }
    }
}

impl BackendNode for ArrayIndexNode {}
