use crate::error::CrocoError;
use crate::symbol_type::SymbolType;
use crate::{
    ast::node::PlusNode,
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
};

impl CrocolNode for PlusNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let left_val = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;
        let right_val = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        match (left_val.symbol_type, right_val.symbol_type) {
            (SymbolType::Num, SymbolType::Num) => {
                let left_float = left_val.value.into_float_value();
                let right_float = right_val.value.into_float_value();

                let res = codegen
                    .builder
                    .build_float_add(left_float, right_float, "tmpadd");

                Ok(LNodeResult::Value(LSymbol {
                    value: res.into(),
                    symbol_type: SymbolType::Num,
                }))
            }

            (SymbolType::Str, SymbolType::Str) => {
                let left_str = codegen.builder.build_alloca(codegen.str_type, "tmpstr");
                codegen.builder.build_store(left_str, left_val.value);

                let left_heap_ptr_ptr = codegen
                    .builder
                    .build_struct_gep(left_str, 0, "gepheapptr")
                    .unwrap();
                let left_heap_ptr = codegen.builder.build_load(left_heap_ptr_ptr, "loadheapptr");
                let left_len_ptr = codegen
                    .builder
                    .build_struct_gep(left_str, 1, "geplen")
                    .unwrap();
                let left_len = codegen.builder.build_load(left_len_ptr, "loadlen");

                let right_str = codegen.builder.build_alloca(codegen.str_type, "tmpstr");
                codegen.builder.build_store(right_str, right_val.value);

                let right_heap_ptr_ptr = codegen
                    .builder
                    .build_struct_gep(right_str, 0, "gepheapptr")
                    .unwrap();
                let right_heap_ptr = codegen
                    .builder
                    .build_load(right_heap_ptr_ptr, "loadheapptr");
                let right_len_ptr = codegen
                    .builder
                    .build_struct_gep(right_str, 1, "geplen")
                    .unwrap();
                let right_len = codegen.builder.build_load(right_len_ptr, "loadlen");

                // get the combined length of both strings
                let combined_length = codegen.builder.build_int_add(
                    left_len.into_int_value(),
                    right_len.into_int_value(),
                    "addlen",
                );

                // create our new string
                let str_type = codegen.module.get_struct_type("struct.CrocoStr").unwrap();
                let alloca = codegen.create_block_alloca(str_type.into(), "allocastr");

                let new_len_ptr = codegen
                    .builder
                    .build_struct_gep(alloca, 1, "geplen")
                    .unwrap();
                codegen.builder.build_store(new_len_ptr, combined_length);

                let new_max_len_ptr = codegen
                    .builder
                    .build_struct_gep(alloca, 2, "gepmaxlen")
                    .unwrap();
                codegen
                    .builder
                    .build_store(new_max_len_ptr, combined_length);

                let new_heap_ptr_ptr = codegen
                    .builder
                    .build_struct_gep(alloca, 0, "gepheapptr")
                    .unwrap();
                let malloc = codegen
                    .builder
                    .build_array_malloc(codegen.context.i8_type(), combined_length, "mallocstr")
                    .unwrap();
                codegen.builder.build_store(new_heap_ptr_ptr, malloc);

                // copy the first str into our new str
                // FOR SOME REASON MEMCPY DOESN'T WORK BUT MEMMOVE WORKS,
                // EVEN IF OUR STRINGS AREN'T OVERLAPPING !!
                codegen
                    .builder
                    .build_memmove(
                        malloc,
                        8,
                        left_heap_ptr.into_pointer_value(),
                        8,
                        left_len.into_int_value(),
                    )
                    .unwrap();

                let malloc_offset = unsafe {
                    codegen
                        .builder
                        .build_gep(malloc, &[left_len.into_int_value()], "gepaddstr")
                };

                // copy the second str
                codegen
                    .builder
                    .build_memmove(
                        malloc_offset,
                        8,
                        right_heap_ptr.into_pointer_value(),
                        8,
                        right_len.into_int_value(),
                    )
                    .unwrap();

                Ok(LNodeResult::Value(LSymbol {
                    value: codegen.builder.build_load(alloca, "loadstradd"),
                    symbol_type: SymbolType::Str,
                }))
            }
            _ => Err(CrocoError::add_error(&self.code_pos)),
        }
    }
}
