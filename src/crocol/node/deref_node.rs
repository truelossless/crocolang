use crate::crocol::{LCodegen, LNodeResult, LSymbol};
use crate::error::CrocoError;
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
            .into_value(&self.code_pos)?;

        let symbol = LSymbol {
            value: codegen
                .builder
                .build_load(symbol.value.into_pointer_value(), "deref"),
            symbol_type: symbol.symbol_type.deref(),
        };

        Ok(LNodeResult::Value(symbol))
    }
}
