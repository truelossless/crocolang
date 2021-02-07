use crate::crocol::CrocolNode;
use crate::{ast::node::DivideNode, error::CrocoError};
use crate::{
    crocol::{LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
};

impl CrocolNode for DivideNode {
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
                    .build_float_div(
                        left.value.into_float_value(),
                        right.value.into_float_value(),
                        "fdiv",
                    )
                    .into(),
                symbol_type: SymbolType::Fnum,
            },

            (SymbolType::Num, SymbolType::Num) => LSymbol {
                value: codegen
                    .builder
                    .build_int_signed_div(
                        left.value.into_int_value(),
                        right.value.into_int_value(),
                        "div",
                    )
                    .into(),
                symbol_type: SymbolType::Num,
            },

            _ => return Err(CrocoError::divide_error(&self.code_pos)),
        };
        Ok(LNodeResult::Value(symbol))
    }
}
