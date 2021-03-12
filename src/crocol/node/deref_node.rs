use crate::crocol::{LCodegen, LNodeResult, LSymbol};
use crate::error::CrocoError;
use crate::symbol_type::SymbolType;
use crate::{ast::node::DerefNode, crocol::CrocolNode};

impl CrocolNode for DerefNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        match symbol.symbol_type {
            SymbolType::Ref(_) => (),
            _ => return Err(CrocoError::dereference_error(&self.code_pos)),
        }

        let deref_symbol = LSymbol {
            value: symbol.value,
            symbol_type: symbol.symbol_type.deref(),
        };

        Ok(LNodeResult::Variable(deref_symbol))
    }
}
