use crate::crocol::CrocolNode;
use crate::symbol_type::SymbolType;
use crate::{ast::node::AsNode, error::CrocoError};

use {
    crate::crocol::{LCodegen, LNodeResult, LSymbol},
    inkwell::IntPredicate,
};

impl CrocolNode for AsNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let val = self
            .bottom
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        let casted = match (val.symbol_type, &self.as_type) {
            (SymbolType::Bool, SymbolType::Bool)
            | (SymbolType::Str, SymbolType::Str)
            | (SymbolType::Num, SymbolType::Num) => {
                return Err(CrocoError::cast_redundant_error(&self.code_pos))
            }

            (SymbolType::Bool, SymbolType::Num) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    one,
                    "cmpbool",
                );

                LSymbol {
                    value: codegen.builder.build_select(cmp, one, zero, "selectnum"),
                    symbol_type: SymbolType::Num,
                }
            }
            (SymbolType::Bool, SymbolType::Str) => {
                let true_str = codegen.alloc_str("true");
                let false_str = codegen.alloc_str("false");

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    codegen.context.bool_type().const_int(1, false),
                    "cmpbool",
                );

                LSymbol {
                    value: codegen
                        .builder
                        .build_select(cmp, true_str, false_str, "selectstr"),
                    symbol_type: SymbolType::Str,
                }
            }

            (SymbolType::Num, SymbolType::Bool) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    codegen.context.bool_type().const_int(0, false),
                    "cmpbool",
                );

                LSymbol {
                    value: codegen.builder.build_select(cmp, zero, one, "selectstr"),
                    symbol_type: SymbolType::Bool,
                }
            }
            (SymbolType::Num, SymbolType::Str) => {
                let num_as_str_fn = codegen.module.get_function("_as_num_str").unwrap();
                let str_res = codegen.alloc_str("").into();
                codegen
                    .builder
                    .build_call(num_as_str_fn, &[val.value, str_res], "callcast");

                LSymbol {
                    symbol_type: SymbolType::Str,
                    value: codegen
                        .builder
                        .build_load(str_res.into_pointer_value(), "loadstrptr"),
                }
            }

            (SymbolType::Str, SymbolType::Num) => {
                let str_as_num_fn = codegen.module.get_function("_as_str_num").unwrap();
                let num_res = codegen
                    .builder
                    .build_call(str_as_num_fn, &[val.value], "callcast")
                    .try_as_basic_value()
                    .left()
                    .unwrap();

                LSymbol {
                    value: num_res,
                    symbol_type: SymbolType::Num,
                }
            }
            (SymbolType::Str, SymbolType::Bool) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let len_ptr = codegen
                    .builder
                    .build_struct_gep(val.value.into_pointer_value(), 1, "geplen")
                    .unwrap();
                let len = codegen.builder.build_load(len_ptr, "loadlen");

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    len.into_int_value(),
                    zero,
                    "cmplen",
                );

                LSymbol {
                    value: codegen.builder.build_select(cmp, zero, one, "selectbool"),
                    symbol_type: SymbolType::Bool,
                }
            }

            _ => {
                return Err(CrocoError::cast_non_primitive_error(&self.code_pos));
            }
        };

        Ok(LNodeResult::Value(casted))
    }
}
