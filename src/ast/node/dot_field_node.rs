use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum};
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

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let mut symbol = self
            .bottom
            .as_mut()
            .unwrap()
            .visit(symtable)?
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
                        format!("no field with the name {}", self.field_name),
                    )
                })?
                .clone(),

            // str fields
            SymbolContent::Primitive(LiteralEnum::Str(Some(_s))) => {
                todo!();
            }

            // num fields
            SymbolContent::Primitive(LiteralEnum::Num(Some(_n))) => {
                todo!();
            }

            // bool fields
            SymbolContent::Primitive(LiteralEnum::Bool(Some(_b))) => {
                todo!();
            }

            // array fields
            SymbolContent::Array(_arr) => {
                todo!();
            }

            // we should never have a reference / empty primitive
            _ => unreachable!(),
        };

        Ok(NodeResult::Symbol(value))
    }
}
