use inkwell::{types::BasicType, AddressSpace, IntPredicate};

use crate::{
    ast::node::ArrayIndexNode,
    crocol::{
        utils::{get_llvm_type, throw_runtime_error},
        CrocolNode, LCodegen, LNodeResult, LSymbol,
    },
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for ArrayIndexNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let index = self
            .index
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?
            .into_num(&self.code_pos)?;

        let array_ptr = self
            .array
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_pointer(codegen, &self.code_pos)?;

        let el_type = match array_ptr.symbol_type {
            SymbolType::Array(el_type) => el_type,
            _ => return Err(CrocoError::wrong_type_indexing(&self.code_pos)),
        };

        let current_fn = codegen.current_fn.unwrap();

        // we may need to cast the i32 index to i64 for comparisons
        let ptr_sized_index = codegen
            .builder
            .build_int_cast(index, codegen.ptr_size, "indexcast");

        // check if the index is negative
        let negative_index_block = codegen.context.append_basic_block(current_fn, "negindex");
        let positive_index_block = codegen.context.append_basic_block(current_fn, "posindex");

        let negative_index_cmp = codegen.builder.build_int_compare(
            IntPredicate::SLT,
            ptr_sized_index,
            codegen.ptr_size.const_zero(),
            "cmpnegindex",
        );

        codegen.builder.build_conditional_branch(
            negative_index_cmp,
            negative_index_block,
            positive_index_block,
        );

        codegen.builder.position_at_end(negative_index_block);
        throw_runtime_error(CrocoError::negative_indexing_error(&self.code_pos), codegen);

        codegen.builder.position_at_end(positive_index_block);

        // check if the index is inferior to the length of the array
        let oob_index_block = codegen.context.append_basic_block(current_fn, "oobindex");
        let ib_index_block = codegen.context.append_basic_block(current_fn, "ibindex");

        let array_len_ptr = codegen
            .builder
            .build_struct_gep(array_ptr.value.into_pointer_value(), 1, "arraylenptr")
            .unwrap();

        let array_len = codegen.builder.build_load(array_len_ptr, "arraylen");

        let oob_index_cmp = codegen.builder.build_int_compare(
            IntPredicate::SGE,
            ptr_sized_index,
            array_len.into_int_value(),
            "cmpoobindex",
        );

        codegen
            .builder
            .build_conditional_branch(oob_index_cmp, oob_index_block, ib_index_block);

        codegen.builder.position_at_end(oob_index_block);
        throw_runtime_error(
            CrocoError::index_out_of_bounds_error(&self.code_pos),
            codegen,
        );

        codegen.builder.position_at_end(ib_index_block);

        // we can finally get the index
        let heap_ptr_ptr = codegen
            .builder
            .build_struct_gep(array_ptr.value.into_pointer_value(), 0, "arrayheapptrptr")
            .unwrap();

        let heap_ptr = codegen.builder.build_load(heap_ptr_ptr, "heapvoidptr");
        let bitcast_ptr = codegen.builder.build_bitcast(
            heap_ptr,
            get_llvm_type(&el_type, codegen).ptr_type(AddressSpace::Generic),
            "heapelptr",
        );

        // SAFETY: we made sure that the pointer is in bounds
        let el_ptr = unsafe {
            codegen
                .builder
                .build_gep(bitcast_ptr.into_pointer_value(), &[index], "elptr")
        };

        Ok(LNodeResult::Variable(LSymbol {
            value: el_ptr.into(),
            symbol_type: *el_type,
        }))
    }
}
