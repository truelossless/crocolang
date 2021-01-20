use crate::{
    ast::node::NotNode,
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for NotNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let true_value = codegen.context.bool_type().const_int(1, false);
        let false_value = codegen.context.bool_type().const_zero();

        let condition = self
            .bottom
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?
            .into_bool()
            .map_err(|_| CrocoError::invert_error(&self.code_pos))?;

        let not_value =
            codegen
                .builder
                .build_select(condition, false_value, true_value, "selectnot");

        Ok(LNodeResult::Value(LSymbol {
            value: not_value,
            symbol_type: SymbolType::Bool,
        }))
    }
}
