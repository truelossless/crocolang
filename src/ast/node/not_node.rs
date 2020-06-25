use crate::ast::{AstNode, AstNodeType, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

#[derive(Clone)]
/// a node used to invert a boolean value
pub struct NotNode {
    bottom: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl NotNode {
    pub fn new(code_pos: CodePos) -> Self {
        NotNode {
            bottom: None,
            code_pos,
        }
    }
}

impl AstNode for NotNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!();
        }
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        match self.bottom.as_mut().unwrap().visit(symtable)? {
            NodeResult::Symbol(Symbol::Primitive(Bool(Some(b)))) => {
                Ok(NodeResult::Symbol(Symbol::Primitive(Bool(Some(!b)))))
            }
            _ => Err(CrocoError::new(
                &self.code_pos,
                "cannot invert something that isn't a boolean".to_owned(),
            )),
        }
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}
