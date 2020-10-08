use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::token::{CodePos, LiteralEnum::*};

#[cfg(feature = "crocoi")]
use crate::crocoi::{utils::get_number_value, INodeResult, ISymTable, ISymbol};

#[derive(Clone)]
pub struct PowerNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl PowerNode {
    pub fn new(code_pos: CodePos) -> Self {
        PowerNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

impl AstNode for PowerNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, symtable, &self.code_pos)?
            .powf(get_number_value(&mut self.right, symtable, &self.code_pos)?));
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
