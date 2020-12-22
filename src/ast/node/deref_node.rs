use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::token::CodePos;

/// A node dereferencing a symbol reference in the symtable to a value
#[derive(Clone)]
pub struct DerefNode {
    pub symbol: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl DerefNode {
    pub fn new(code_pos: CodePos) -> Self {
        DerefNode {
            symbol: None,
            code_pos,
        }
    }
}

impl AstNode for DerefNode {
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

impl BackendNode for DerefNode {}
