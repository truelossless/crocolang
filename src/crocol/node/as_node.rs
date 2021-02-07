use inkwell::FloatPredicate;

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
            | (SymbolType::Num, SymbolType::Num)
            | (SymbolType::Fnum, SymbolType::Fnum) => {
                return Err(CrocoError::cast_redundant_error(&self.code_pos))
            }

            (SymbolType::Bool, SymbolType::Fnum) => {
                let zero = codegen.context.f32_type().const_zero();
                let one = codegen.context.f32_type().const_float(1.);

                LSymbol {
                    value: codegen.builder.build_select(
                        val.value.into_int_value(),
                        one,
                        zero,
                        "boolfnumcast",
                    ),
                    symbol_type: SymbolType::Fnum,
                }
            }

            (SymbolType::Bool, SymbolType::Num) => {
                let zero = codegen.context.i32_type().const_zero();
                let one = codegen.context.i32_type().const_int(1, true);

                LSymbol {
                    value: codegen.builder.build_select(
                        val.value.into_int_value(),
                        one,
                        zero,
                        "boolnumcast",
                    ),
                    symbol_type: SymbolType::Fnum,
                }
            }

            (SymbolType::Bool, SymbolType::Str) => {
                let true_str_ptr = codegen.alloc_str("true");
                let true_str = codegen.builder.build_load(true_str_ptr, "loadstr");

                let false_str_ptr = codegen.alloc_str("false");
                let false_str = codegen.builder.build_load(false_str_ptr, "loadstr");

                LSymbol {
                    value: codegen.builder.build_select(
                        val.value.into_int_value(),
                        true_str,
                        false_str,
                        "castboolstr",
                    ),
                    symbol_type: SymbolType::Str,
                }
            }

            (SymbolType::Num, SymbolType::Fnum) => {
                let cast = codegen.builder.build_signed_int_to_float(
                    val.value.into_int_value(),
                    codegen.context.f32_type(),
                    "castnumfnum",
                );
                LSymbol {
                    value: cast.into(),
                    symbol_type: SymbolType::Fnum,
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

            (SymbolType::Num, SymbolType::Bool) => {
                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    codegen.context.i32_type().const_zero(),
                    "cmpnum",
                );

                let cast = codegen.builder.build_select(
                    cmp,
                    codegen.context.bool_type().const_zero(),
                    codegen.context.bool_type().const_int(1, false),
                    "castnumbool",
                );

                LSymbol {
                    value: cast,
                    symbol_type: SymbolType::Bool,
                }
            }

            (SymbolType::Fnum, SymbolType::Num) => {
                let cast = codegen.builder.build_float_to_signed_int(
                    val.value.into_float_value(),
                    codegen.context.i32_type(),
                    "castfnumnum",
                );

                LSymbol {
                    value: cast.into(),
                    symbol_type: SymbolType::Num,
                }
            }

            (SymbolType::Fnum, SymbolType::Bool) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let cmp = codegen.builder.build_float_compare(
                    FloatPredicate::UEQ,
                    val.value.into_float_value(),
                    codegen.context.f32_type().const_zero(),
                    "cmpfnumbool",
                );

                LSymbol {
                    value: codegen.builder.build_select(cmp, zero, one, "castfnumbool"),
                    symbol_type: SymbolType::Bool,
                }
            }
            (SymbolType::Fnum, SymbolType::Str) => {
                let fnum_as_str_fn = codegen.module.get_function("_as_fnum_str").unwrap();
                let str_res = codegen.alloc_str("").into();
                codegen
                    .builder
                    .build_call(fnum_as_str_fn, &[val.value, str_res], "callcast");

                LSymbol {
                    symbol_type: SymbolType::Str,
                    value: codegen
                        .builder
                        .build_load(str_res.into_pointer_value(), "loadstrptr"),
                }
            }

            (SymbolType::Str, SymbolType::Fnum) => {
                let str_as_fnum_fn = codegen.module.get_function("_as_str_fnum").unwrap();
                let fnum_res = codegen
                    .builder
                    .build_call(str_as_fnum_fn, &[val.value], "callcast")
                    .try_as_basic_value()
                    .left()
                    .unwrap();

                LSymbol {
                    value: fnum_res,
                    symbol_type: SymbolType::Fnum,
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
