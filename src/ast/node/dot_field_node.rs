use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::token::{CodePos, LiteralEnum};

#[cfg(feature = "crocoi")]
use crate::crocoi::{utils::auto_deref, ISymTable, ISymbol};

/// a node to access symbol fields
#[derive(Clone)]
pub struct DotFieldNode {
    field_name: String,
    bottom: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl DotFieldNode {
    pub fn new(field_name: String, code_pos: CodePos) -> Self {
        DotFieldNode {
            bottom: None,
            field_name,
            code_pos,
        }
    }
}

impl AstNode for DotFieldNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!()
        }
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        
        let mut symbol = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_symbol(&self.code_pos)?;
            
        symbol = auto_deref(symbol);

        let value = match &symbol {
            // access a struct field
            ISymbol::Struct(s) => s
                .fields
                .get(&self.field_name)
                .ok_or_else(|| {
                    CrocoError::new(
                        &self.code_pos,
                        format!("no field with the name {}", self.field_name),
                    )
                })?
                .clone(),

            // str fields
            ISymbol::Primitive(LiteralEnum::Str(_s)) => {
                todo!();
            }

            // num fields
            ISymbol::Primitive(LiteralEnum::Num(_n)) => {
                todo!();
            }

            // bool fields
            ISymbol::Primitive(LiteralEnum::Bool(_b)) => {
                todo!();
            }

            // array fields
            ISymbol::Array(_arr) => {
                todo!();
            }

            // we should never have a reference / empty primitive
            _ => unreachable!(),
        };
        Ok(INodeResult::Variable(value))
    }
}
