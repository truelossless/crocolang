use crate::ast::{AstNode, AstNodeType, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum::*};

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

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let val = self
            .left
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)?;

        let as_type = self
            .right
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)?;

        // we can only cast primitive together
        let val_primitive = val.borrow().clone().into_primitive().map_err(|_| {
            CrocoError::new(
                &self.code_pos,
                "can only cast primitives together".to_owned(),
            )
        })?;

        let as_type_primitive = as_type.borrow().clone().into_primitive().map_err(|_| {
            CrocoError::new(
                &self.code_pos,
                "can only cast primitives together".to_owned(),
            )
        })?;

        let casted = match (val_primitive, as_type_primitive) {
            // useless cast
            (Bool(_), Bool(_)) | (Str(_), Str(_)) | (Num(_), Num(_)) => {
                return Err(CrocoError::new(&self.code_pos, "redundant cast".to_owned()))
            }

            (Bool(Some(b)), Num(_)) => {
                if b {
                    Num(Some(1.))
                } else {
                    Num(Some(0.))
                }
            }
            (Bool(Some(b)), Str(_)) => {
                if b {
                    Str(Some("true".to_owned()))
                } else {
                    Str(Some("false".to_owned()))
                }
            }

            (Num(Some(n)), Bool(_)) => {
                if n == 0. {
                    Bool(Some(false))
                } else {
                    Bool(Some(true))
                }
            }
            (Num(Some(n)), Str(_)) => Str(Some(n.to_string())),

            (Str(Some(s)), Num(_)) => {
                let n = s.parse().map_err(|_| {
                    CrocoError::new(
                        &self.code_pos,
                        "could not parse the str into a num".to_owned(),
                    )
                })?;
                Num(Some(n))
            }
            (Str(Some(s)), Bool(_)) => {
                if s == "true" {
                    Bool(Some(true))
                } else {
                    Bool(Some(false))
                }
            }

            _ => unreachable!(),
        };

        Ok(NodeResult::construct_symbol(SymbolContent::Primitive(
            casted,
        )))
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
