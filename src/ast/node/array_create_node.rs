use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;

/// A node representing an array symbol
/// checks at runtime if the type constraint is respected
#[derive(Clone)]
pub struct ArrayCreateNode {
    pub contents: Vec<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl ArrayCreateNode {
    pub fn new(contents: Vec<Box<dyn BackendNode>>, code_pos: CodePos) -> Self {
        ArrayCreateNode { contents, code_pos }
    }
}

impl AstNode for ArrayCreateNode {
    fn prepend_child(&mut self, _node: Box<dyn BackendNode>) {
        unimplemented!();
    }

    fn add_child(&mut self, _node: Box<dyn BackendNode>) {
        unimplemented!();
    }

    fn get_type(&self) -> crate::ast::AstNodeType {
        unimplemented!();
    }
}

impl BackendNode for ArrayCreateNode {}
