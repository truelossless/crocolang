use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::token::{CodePos, LiteralEnum::*};

#[cfg(feature = "crocoi")]
use crate::crocoi::symbol::{ISymTable, ISymbol};

#[derive(Clone)]
/// a node used to invert a boolean value
pub struct NotNode {
    bottom: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl NotNode {
    pub fn new(code_pos: CodePos) -> Self {
        NotNode {
            bottom: None,
            code_pos,
        }
    }
}

impl AstNode for NotNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!();
        }
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let bool_symbol = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_symbol(&self.code_pos)?;

        let condition = bool_symbol
            .into_primitive()
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "cannot invert something that isn't a boolean",
                )
            })?
            .into_bool()
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "cannot invert something that isn't a boolean",
                )
            })?;

        Ok(INodeResult::Value(ISymbol::Primitive(Bool(!condition))))
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}
