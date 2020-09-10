use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    crocoi::{symbol::SymbolContent, ISymbol},
    token::{CodePos, LiteralEnum},
};
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

    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let mut symbol = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_symbol(&self.code_pos)?;

        // auto deref if we have a Ref
        loop {
            let reference;

            if let SymbolContent::Ref(r) = &*symbol.borrow() {
                reference = r.clone();
            } else {
                break;
            }

            symbol = reference;
        }

        let err = format!("no field with the name {}", 12);
        let value = match &*symbol.borrow() {
            // access a struct field



            SymbolContent::Struct(s) => s
                .fields
                .as_ref()
                .unwrap()
                .get(&self.field_name)
                .ok_or_else(|| {
                    CrocoError::new(
                        &self.code_pos,
                        // &format!("no field with the name {}", self.field_name),
                        &err
                    )
                })?
                .clone(),

            // str fields
            SymbolContent::Primitive(LiteralEnum::Str(_s)) => {
                todo!();
            }

            // num fields
            SymbolContent::Primitive(LiteralEnum::Num(_n)) => {
                todo!();
            }

            // bool fields
            SymbolContent::Primitive(LiteralEnum::Bool(_b)) => {
                todo!();
            }

            // array fields
            SymbolContent::Array(_arr) => {
                todo!();
            }

            // we should never have a reference / empty primitive
            _ => unreachable!(),
        };

        Ok(INodeResult::Symbol(value))
    }
}
