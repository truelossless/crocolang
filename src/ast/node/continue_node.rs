use crate::ast::{AstNode, BackendNode};

/// A node representing a continue statement
#[derive(Clone)]
pub struct ContinueNode {}

impl ContinueNode {
    pub fn new() -> Self {
        ContinueNode {}
    }
}

impl AstNode for ContinueNode {}
impl BackendNode for ContinueNode {}
