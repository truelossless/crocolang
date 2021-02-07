use crate::{
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
};

use crate::ast::node::MultiplicateNode;
use crate::error::CrocoError;

impl CrocolNode for MultiplicateNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let left = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        let right = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        let symbol = match (left.symbol_type, right.symbol_type) {
            (SymbolType::Fnum, SymbolType::Fnum) => LSymbol {
                value: codegen
                    .builder
                    .build_float_mul(
                        left.value.into_float_value(),
                        right.value.into_float_value(),
                        "fmul",
                    )
                    .into(),
                symbol_type: SymbolType::Fnum,
            },

            (SymbolType::Num, SymbolType::Num) => LSymbol {
                value: codegen
                    .builder
                    .build_int_mul(
                        left.value.into_int_value(),
                        right.value.into_int_value(),
                        "mul",
                    )
                    .into(),
                symbol_type: SymbolType::Num,
            },

            _ => return Err(CrocoError::minus_error(&self.code_pos)),
        };
        Ok(LNodeResult::Value(symbol))
    }
}
