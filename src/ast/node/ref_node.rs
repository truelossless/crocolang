use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;
/// a node creating a reference to a symbol
#[derive(Clone)]
pub struct RefNode {
    pub symbol: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl RefNode {
    pub fn new(code_pos: CodePos) -> Self {
        RefNode {
            symbol: None,
            code_pos,
        }
    }
}

impl AstNode for RefNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.symbol.is_none() {
            self.symbol = Some(node);
        } else {
            unreachable!()
        }
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}

impl BackendNode for RefNode {}
