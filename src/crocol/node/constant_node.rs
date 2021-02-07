use crate::{ast::node::ConstantNode, crocol::CrocolNode, token::LiteralEnum, CrocoError};

use {
    crate::crocol::{LCodegen, LNodeResult, LSymbol},
    crate::symbol_type::SymbolType,
};

impl CrocolNode for ConstantNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let constant_symbol = match &self.value {
            LiteralEnum::Bool(b) => LSymbol {
                value: codegen
                    .context
                    .bool_type()
                    .const_int(*b as u64, false)
                    .into(),
                symbol_type: SymbolType::Bool,
            },

            LiteralEnum::Num(n) => LSymbol {
                value: codegen.context.i32_type().const_int(*n as u64, true).into(),
                symbol_type: SymbolType::Num,
            },

            LiteralEnum::Fnum(n) => LSymbol {
                value: codegen.context.f32_type().const_float(*n as f64).into(),
                symbol_type: SymbolType::Fnum,
            },

            LiteralEnum::Str(s) => {
                let alloca = codegen.alloc_str(s);
                let load = codegen.builder.build_load(alloca, "loadstr");

                LSymbol {
                    value: load,
                    symbol_type: SymbolType::Str,
                }
            }
        };

        Ok(LNodeResult::Value(constant_symbol))
    }
}
