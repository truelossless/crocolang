use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{crocoi::{utils::get_number_value, ISymbol, symbol::SymbolContent}, token::{CodePos, LiteralEnum::*}};

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

    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, symtable, &self.code_pos)?
            .powf(get_number_value(&mut self.right, symtable, &self.code_pos)?));
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            value,
        )))
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
