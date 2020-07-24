use crate::ast::utils::get_value;
use crate::ast::{AstNode, AstNodeType, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum::*};

/// a node used for addition and concatenation
#[derive(Clone)]
pub struct PlusNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl PlusNode {
    pub fn new(code_pos: CodePos) -> Self {
        PlusNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

// TODO: remove implicit cast and introduce as keyword
// TODO: put all math nodes together ?
/// node handling additions and concatenations
impl AstNode for PlusNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, symtable, &self.code_pos)?;
        let right_val = get_value(&mut self.right, symtable, &self.code_pos)?;

        // different kinds of additions can happen (concatenation or number addition)
        // the PlusNode also works for concatenation.
        let value = match (left_val, right_val) {
            (Num(Some(n1)), Num(Some(n2))) => Num(Some(n1 + n2)),
            (Str(Some(s1)), Str(Some(s2))) => Str(Some(format!("{}{}", s1, s2))),
            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "cannot add these two types together".to_owned(),
                ))
            }
        };
        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(value)))
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
