#[cfg(feature = "crocoi")]
use crate::crocoi::{utils::get_number_value, INodeResult, ISymTable, ISymbol};

#[cfg(feature = "crocol")]
use crate::{
    crocol::{Codegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
};

use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::token::{CodePos, LiteralEnum::*};
#[derive(Clone)]
pub struct MultiplicateNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl MultiplicateNode {
    pub fn new(code_pos: CodePos) -> Self {
        MultiplicateNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

impl AstNode for MultiplicateNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, symtable, &self.code_pos)?
            * get_number_value(&mut self.right, symtable, &self.code_pos)?);
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let left_val = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_value(&self.code_pos)?;
        let right_val = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_value(&self.code_pos)?;

        let left_float = left_val.value.into_float_value();
        let right_float = right_val.value.into_float_value();

        let res = codegen
            .builder
            .build_float_mul(left_float, right_float, "tmpdiv");

        let symbol = LSymbol {
            value: res.into(),
            symbol_type: SymbolType::Num,
        };

        Ok(LNodeResult::Value(symbol))
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
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
