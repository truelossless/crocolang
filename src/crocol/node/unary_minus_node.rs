use crate::{
    ast::node::UnaryMinusNode,
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for UnaryMinusNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let bottom = self
            .bottom
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        let symbol = match bottom.symbol_type {
            SymbolType::Fnum => LSymbol {
                value: codegen
                    .builder
                    .build_float_sub(
                        codegen.context.f32_type().const_zero(),
                        bottom.value.into_float_value(),
                        "unaryfsub",
                    )
                    .into(),
                symbol_type: SymbolType::Fnum,
            },

            SymbolType::Num => LSymbol {
                value: codegen
                    .builder
                    .build_int_sub(
                        codegen.context.i32_type().const_zero(),
                        bottom.value.into_int_value(),
                        "unarysub",
                    )
                    .into(),
                symbol_type: SymbolType::Num,
            },

            _ => return Err(CrocoError::unary_minus_error(&self.code_pos)),
        };

        Ok(LNodeResult::Value(symbol))
    }
}
