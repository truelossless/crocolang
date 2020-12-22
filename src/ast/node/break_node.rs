use crate::ast::{AstNode, BackendNode};

/// A node representing a break statement
#[derive(Clone)]
pub struct BreakNode {}

impl BreakNode {
    pub fn new() -> Self {
        BreakNode {}
    }
}

impl AstNode for BreakNode {}
impl BackendNode for BreakNode {}
