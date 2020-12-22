use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;

#[derive(Clone)]
pub struct UnaryMinusNode {
    pub bottom: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl UnaryMinusNode {
    pub fn new(code_pos: CodePos) -> Self {
        UnaryMinusNode {
            bottom: None,
            code_pos,
        }
    }
}

impl AstNode for UnaryMinusNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!()
        }
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}

impl BackendNode for UnaryMinusNode {}
