use crate::{
    ast::node::RefNode,
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for RefNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_var(&self.code_pos)
            .map_err(|_| CrocoError::tmp_value_borrow(&self.code_pos))?;

        let ref_symbol = LSymbol {
            value: symbol.value,
            symbol_type: SymbolType::Ref(Box::new(symbol.symbol_type)),
        };

        Ok(LNodeResult::Value(ref_symbol))
    }
}
