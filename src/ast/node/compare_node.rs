use crate::ast::utils::get_value;
use crate::ast::{AstNode, AstNodeType, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent};
use crate::token::{literal_eq, CodePos, LiteralEnum::*, OperatorEnum};

#[derive(Clone)]
/// A node used to compare two values, returns a boolean
pub struct CompareNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    compare_kind: OperatorEnum,
    code_pos: CodePos,
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
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, symtable, &self.code_pos)?;
        let right_val = get_value(&mut self.right, symtable, &self.code_pos)?;

        if !literal_eq(&left_val, &right_val) {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot compare different types".to_owned(),
            ));
        }

        if (self.compare_kind != OperatorEnum::Equals
            || self.compare_kind == OperatorEnum::NotEquals)
            && !left_val.is_num()
        {
            return Err(CrocoError::new(
                &self.code_pos,
                "can compare only numbers".to_owned(),
            ));
        }

        let value = match self.compare_kind {
            OperatorEnum::Equals => left_val == right_val,
            OperatorEnum::NotEquals => left_val != right_val,
            OperatorEnum::GreaterOrEqual => left_val >= right_val,
            OperatorEnum::GreaterThan => left_val > right_val,
            OperatorEnum::LowerOrEqual => left_val <= right_val,
            OperatorEnum::LowerThan => left_val < right_val,
            _ => unreachable!(),
        };

        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(
            Bool(Some(value)),
        )))
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
