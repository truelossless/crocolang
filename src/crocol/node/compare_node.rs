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

        let bool_res = match left_value.symbol_type {
            SymbolType::Num => {
                let op = match self.compare_kind {
                    OperatorEnum::Equals => FloatPredicate::OEQ,
                    OperatorEnum::NotEquals => FloatPredicate::ONE,
                    OperatorEnum::GreaterThan => FloatPredicate::OGT,
                    OperatorEnum::GreaterOrEqual => FloatPredicate::OGE,
                    OperatorEnum::LowerThan => FloatPredicate::OLT,
                    OperatorEnum::LowerOrEqual => FloatPredicate::OLE,
                    _ => unreachable!(),
                };

                codegen.builder.build_float_compare(
                    op,
                    left_value.value.into_float_value(),
                    right_value.value.into_float_value(),
                    "cmpnum",
                )
            }

            SymbolType::Bool => {
                let op = match self.compare_kind {
                    OperatorEnum::Equals => IntPredicate::EQ,
                    OperatorEnum::NotEquals => IntPredicate::NE,
                    _ => return Err(CrocoError::compare_numbers_only_error(&self.code_pos)),
                };

                codegen.builder.build_int_compare(
                    op,
                    left_value.value.into_int_value(),
                    right_value.value.into_int_value(),
                    "cmpbool",
                )
            }

            SymbolType::Str => {
                let cmp_fn = codegen.module.get_function("_croco_str_cmp").unwrap();

                let left_ptr = codegen.create_block_alloca(codegen.str_type.into(), "tmpstr");
                let right_ptr = codegen.create_block_alloca(codegen.str_type.into(), "tmpstr");

                codegen.builder.build_store(left_ptr, left_value.value);
                codegen.builder.build_store(right_ptr, right_value.value);

                let cmp_res = codegen
                    .builder
                    .build_call(cmp_fn, &[left_ptr.into(), right_ptr.into()], "cmpstr")
                    .try_as_basic_value()
                    .left()
                    .unwrap();

                let op = match self.compare_kind {
                    OperatorEnum::Equals => IntPredicate::EQ,
                    OperatorEnum::NotEquals => IntPredicate::NE,
                    OperatorEnum::GreaterThan => IntPredicate::SGT,
                    OperatorEnum::GreaterOrEqual => IntPredicate::SGE,
                    OperatorEnum::LowerThan => IntPredicate::SLT,
                    OperatorEnum::LowerOrEqual => IntPredicate::SLE,
                    _ => unreachable!(),
                };

                codegen.builder.build_int_compare(
                    op,
                    cmp_res.into_int_value(),
                    codegen.context.i8_type().const_zero(),
                    "cmpstr",
                )
            }

            _ => unimplemented!(),
        };

        Ok(LNodeResult::Value(LSymbol {
            symbol_type: SymbolType::Bool,
            value: bool_res.into(),
        }))
    }
}
