use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;

/// A node returning a value from a block
#[derive(Clone)]
pub struct ReturnNode {
    pub bottom: Box<dyn BackendNode>,
    pub code_pos: CodePos,
}

impl ReturnNode {
    pub fn new(bottom: Box<dyn BackendNode>, code_pos: CodePos) -> Self {
        ReturnNode { bottom, code_pos }
    }
}

impl AstNode for ReturnNode {}
impl BackendNode for ReturnNode {}
