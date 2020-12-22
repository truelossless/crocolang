use crate::{
    ast::{AstNode, BackendNode},
    token::{CodePos, LiteralEnum},
};

/// A node holding a literal value
#[derive(Clone)]
pub struct ConstantNode {
    pub value: LiteralEnum,
    pub code_pos: CodePos,
}

impl ConstantNode {
    pub fn new(value: LiteralEnum, code_pos: CodePos) -> Self {
        ConstantNode { value, code_pos }
    }
}

impl AstNode for ConstantNode {}
impl BackendNode for ConstantNode {}
