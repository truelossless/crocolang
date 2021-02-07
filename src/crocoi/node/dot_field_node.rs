use crate::crocoi::{utils::auto_deref, ICodegen, INodeResult, ISymbol};
use crate::error::CrocoError;
use crate::token::LiteralEnum;
use crate::{ast::node::DotFieldNode, crocoi::CrocoiNode};

impl CrocoiNode for DotFieldNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let mut symbol = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(codegen)?
            .into_symbol(&self.code_pos)?;

        symbol = auto_deref(symbol);

        let value = match &symbol {
            // access a struct field
            ISymbol::Struct(s) => s
                .fields
                .get(&self.field_name)
                .ok_or_else(|| CrocoError::no_field_error(&self.field_name, &self.code_pos))?
                .clone(),

            // str fields
            ISymbol::Primitive(LiteralEnum::Str(_s)) => {
                todo!();
            }

            // num fields
            ISymbol::Primitive(LiteralEnum::Fnum(_n)) => {
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
