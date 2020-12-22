use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;

/// a node representing an if / elif / else structure
#[derive(Clone)]
pub struct IfNode {
    // comparison value (a CompareNode)
    pub conditions: Vec<Box<dyn BackendNode>>,
    // if / elif / else bodies (a BlockNode)
    pub bodies: Vec<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl IfNode {
    pub fn new(
        conditions: Vec<Box<dyn BackendNode>>,
        bodies: Vec<Box<dyn BackendNode>>,
        code_pos: CodePos,
    ) -> Self {
        IfNode {
            conditions,
            bodies,
            code_pos,
        }
    }
}

impl AstNode for IfNode {}
impl BackendNode for IfNode {}
