use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{crocoi::{utils::get_value, ISymbol, symbol::SymbolContent}, token::{CodePos, LiteralEnum::*}};

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

/// node handling additions and concatenations
impl AstNode for PlusNode {
    fn visit(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, symtable, &self.code_pos)?;
        let right_val = get_value(&mut self.right, symtable, &self.code_pos)?;

        // different kinds of additions can happen (concatenation or number addition)
        // the PlusNode also works for concatenation.
        let value = match (left_val, right_val) {
            (Num(n1), Num(n2)) => Num(n1 + n2),
            (Str(s1), Str(s2)) => Str(format!("{}{}", s1, s2)),
            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "cannot add these two types together",
                ))
            }
        };
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            value,
        )))
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
