use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, LiteralEnum::*}, crocoi::{ISymbol, INodeResult, symbol::SymbolContent},
};

#[derive(Clone)]
/// a node used to cast primitives
pub struct AsNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl AsNode {
    pub fn new(code_pos: CodePos) -> Self {
        AsNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

impl AstNode for AsNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!();
        }
    }

    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let val = self
            .left
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_symbol(&self.code_pos)?;

        // TODO: don't clone and take a ref
        let as_type = self
            .right
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_symbol(&self.code_pos)?
            .borrow()
            .clone()
            .into_croco_type()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        // we can only cast primitive together
        let val_primitive =
            val.borrow().clone().into_primitive().map_err(|_| {
                CrocoError::new(&self.code_pos, "can only cast primitives together")
            })?;

        let casted = match (val_primitive, as_type) {
            // useless cast
            (Bool(_), SymbolType::Bool) | (Str(_), SymbolType::Str) | (Num(_), SymbolType::Num) => {
                return Err(CrocoError::new(&self.code_pos, "redundant cast"))
            }

            (Bool(b), SymbolType::Num) => {
                if b {
                    Num(1.)
                } else {
                    Num(0.)
                }
            }
            (Bool(b), SymbolType::Str) => {
                if b {
                    Str("true".to_owned())
                } else {
                    Str("false".to_owned())
                }
            }

            (Num(n), SymbolType::Bool) => {
                if n == 0. {
                    Bool(false)
                } else {
                    Bool(true)
                }
            }
            (Num(n), SymbolType::Str) => Str(n.to_string()),

            (Str(s), SymbolType::Num) => {
                let n = s.parse().map_err(|_| {
                    CrocoError::new(&self.code_pos, "could not parse the str into a num")
                })?;
                Num(n)
            }
            (Str(s), SymbolType::Bool) => {
                if !s.is_empty() {
                    Bool(true)
                } else {
                    Bool(false)
                }
            }

            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "only primitives can be casted together",
                ))
            }
        };

        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            casted,
        )))
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
