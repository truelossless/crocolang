use inkwell::{FloatPredicate, IntPredicate};

use crate::{
    ast::node::CompareNode,
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
    token::OperatorEnum,
    CrocoError,
};

impl CrocolNode for CompareNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let left_value = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        let right_value = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        if left_value.symbol_type != right_value.symbol_type {
            return Err(CrocoError::compare_different_types_error(&self.code_pos));
        }

        if (self.compare_kind != OperatorEnum::Equals
            && self.compare_kind != OperatorEnum::NotEquals)
            && left_value.symbol_type != SymbolType::Num
        {
            return Err(CrocoError::compare_numbers_only_error(&self.code_pos));
        }

        let bool_res = match self.compare_kind {
            OperatorEnum::Equals | OperatorEnum::NotEquals => match left_value.symbol_type {
                SymbolType::Num => {
                    if self.compare_kind == OperatorEnum::Equals {
                        codegen.builder.build_float_compare(
                            FloatPredicate::UEQ,
                            left_value.value.into_float_value(),
                            right_value.value.into_float_value(),
                            "cmpnumeq",
                        )
                    } else {
                        codegen.builder.build_float_compare(
                            FloatPredicate::UNE,
                            left_value.value.into_float_value(),
                            right_value.value.into_float_value(),
                            "cmpnumne",
                        )
                    }
                }

                SymbolType::Bool => {
                    if self.compare_kind == OperatorEnum::Equals {
                        codegen.builder.build_int_compare(
                            inkwell::IntPredicate::EQ,
                            left_value.value.into_int_value(),
                            right_value.value.into_int_value(),
                            "cmpbooleq",
                        )
                    } else {
                        codegen.builder.build_int_compare(
                            IntPredicate::NE,
                            left_value.value.into_int_value(),
                            right_value.value.into_int_value(),
                            "cmpboolne",
                        )
                    }
                }

                SymbolType::Str => {
                    let cmp_fn = codegen.module.get_function("_croco_str_cmp").unwrap();
                    let cmp_res = codegen
                        .builder
                        .build_call(cmp_fn, &[left_value.value, right_value.value], "cmpstr")
                        .try_as_basic_value()
                        .left()
                        .unwrap();

                    if self.compare_kind == OperatorEnum::Equals {
                        codegen.builder.build_int_compare(
                            IntPredicate::EQ,
                            codegen.context.i8_type().const_zero(),
                            cmp_res.into_int_value(),
                            "cmpstreq",
                        )
                    } else {
                        codegen.builder.build_int_compare(
                            IntPredicate::NE,
                            codegen.context.i8_type().const_zero(),
                            cmp_res.into_int_value(),
                            "cmpstrne",
                        )
                    }
                }

                _ => return Err(CrocoError::compare_numbers_only_error(&self.code_pos)),
            },

            _ => unimplemented!(),
        };

        Ok(LNodeResult::Value(LSymbol {
            symbol_type: SymbolType::Bool,
            value: bool_res.into(),
        }))
    }
}
