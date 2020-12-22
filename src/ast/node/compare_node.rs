use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;
use crate::token::OperatorEnum;

#[derive(Clone)]
/// A node used to compare two values, returns a boolean
pub struct CompareNode {
    pub left: Option<Box<dyn BackendNode>>,
    pub right: Option<Box<dyn BackendNode>>,
    pub compare_kind: OperatorEnum,
    pub code_pos: CodePos,
}

impl CompareNode {
    pub fn new(compare_kind: OperatorEnum, code_pos: CodePos) -> Self {
        CompareNode {
            left: None,
            right: None,
            compare_kind,
            code_pos,
        }
    }
}

impl AstNode for CompareNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}

impl BackendNode for CompareNode {}
