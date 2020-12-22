use crate::ast::{AstNode, BackendNode};

/// A node returning a Void NodeResult
#[derive(Clone)]
pub struct VoidNode {}

impl VoidNode {
    pub fn new() -> Self {
        VoidNode {}
    }
}

impl AstNode for VoidNode {}
impl BackendNode for VoidNode {}
