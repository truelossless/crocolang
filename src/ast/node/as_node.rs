use crate::ast::{AstNode, AstNodeType, BackendNode};
use crate::{symbol_type::SymbolType, token::CodePos};

#[derive(Clone)]
/// A node used to cast primitives
pub struct AsNode {
    pub bottom: Option<Box<dyn BackendNode>>,
    pub as_type: SymbolType,
    pub code_pos: CodePos,
}

impl AsNode {
    pub fn new(as_type: SymbolType, code_pos: CodePos) -> Self {
        AsNode {
            bottom: None,
            as_type,
            code_pos,
        }
    }
}

impl AstNode for AsNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!();
        }
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}

impl BackendNode for AsNode {}
