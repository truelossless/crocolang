use crate::crocoi::CrocoiNode;
use crate::{ast::node::NotNode, error::CrocoError};
use crate::{crocoi::INodeResult, token::LiteralEnum::*};

use crate::crocoi::symbol::{ICodegen, ISymbol};

impl CrocoiNode for NotNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let bool_symbol = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(codegen)?
            .into_symbol(&self.code_pos)?;

        let condition = bool_symbol
            .into_primitive()
            .map_err(|_| CrocoError::invert_error(&self.code_pos))?
            .into_bool()
            .map_err(|_| CrocoError::invert_error(&self.code_pos))?;

        Ok(INodeResult::Value(ISymbol::Primitive(Bool(!condition))))
    }
}
