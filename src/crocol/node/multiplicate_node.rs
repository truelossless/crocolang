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
        let left_val = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?
            .into_num(&self.code_pos)?;

        let right_val = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?
            .into_num(&self.code_pos)?;

        let res = codegen.builder.build_float_mul(left_val, right_val, "mul");

        let symbol = LSymbol {
            value: res.into(),
            symbol_type: SymbolType::Num,
        };

        Ok(LNodeResult::Value(symbol))
    }
}
