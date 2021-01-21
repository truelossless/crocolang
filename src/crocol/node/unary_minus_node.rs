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
        let value = self
            .bottom
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?
            .into_num(&self.code_pos)?;

        let res = codegen.builder.build_float_sub(
            codegen.context.f32_type().const_zero(),
            value,
            "unarysub",
        );

        let res = LSymbol {
            value: res.into(),
            symbol_type: SymbolType::Num,
        };

        Ok(LNodeResult::Value(res))
    }
}
