use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::token::{literal_eq, CodePos, LiteralEnum::*, OperatorEnum};

#[cfg(feature = "crocoi")]
use crate::crocoi::{utils::get_value, INodeResult, ISymTable, ISymbol};

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

    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, symtable, &self.code_pos)?;
        let right_val = get_value(&mut self.right, symtable, &self.code_pos)?;

        if !literal_eq(&left_val, &right_val) {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot compare different types",
            ));
        }

        if (self.compare_kind != OperatorEnum::Equals
            || self.compare_kind == OperatorEnum::NotEquals)
            && !left_val.is_num()
        {
            return Err(CrocoError::new(&self.code_pos, "can compare only numbers"));
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

        Ok(INodeResult::Value(ISymbol::Primitive(Bool(value))))
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
