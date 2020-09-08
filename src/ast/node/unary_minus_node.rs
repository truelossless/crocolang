use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    crocoi::{symbol::SymbolContent, utils::get_number_value, ISymbol},
    token::{CodePos, LiteralEnum::*},
};
#[derive(Clone)]
pub struct UnaryMinusNode {
    bottom: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
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
    fn visit(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let value = Num(-get_number_value(
            &mut self.bottom,
            symtable,
            &self.code_pos,
        )?);
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            value,
        )))
    }
    fn add_child(&mut self, node: Box<dyn AstNode>) {
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
