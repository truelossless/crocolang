use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::{
    crocoi::{utils::get_number_value, ISymTable, ISymbol},
    token::LiteralEnum::*,
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
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let value = Num(-get_number_value(
            &mut self.bottom,
            symtable,
            &self.code_pos,
        )?);
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
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
