use crate::{
    crocol::{Codegen, LSymbol},
    error::CrocoError,
    error::CrocoErrorKind,
    symbol_type::SymbolType,
};

use inkwell::{
    context::Context,
    types::IntType,
    types::{AnyTypeEnum, BasicType, BasicTypeEnum, StructType},
    values::PointerValue,
    AddressSpace, IntPredicate,
};
use std::{convert::TryInto, path::Path};

/// transforms a AnyTypeEnum in a ptr of an AnyTypeEnum
// not the most beautiful code, but eh it works
pub fn any_type_ptr(any_type: AnyTypeEnum<'_>) -> AnyTypeEnum<'_> {
    match any_type {
        AnyTypeEnum::ArrayType(a) => a.ptr_type(AddressSpace::Generic).into(),
        AnyTypeEnum::FloatType(f) => f.ptr_type(AddressSpace::Generic).into(),
        AnyTypeEnum::StructType(s) => s.ptr_type(AddressSpace::Generic).into(),
        AnyTypeEnum::PointerType(p) => p.ptr_type(AddressSpace::Generic).into(),
        _ => unreachable!(),
    }
}

/// get the llvm type corresponding to a SymbolType
pub fn get_llvm_type<'ctx>(symbol_type: &SymbolType, codegen: &Codegen<'ctx>) -> AnyTypeEnum<'ctx> {
    match symbol_type {
        SymbolType::Num => codegen.context.f32_type().into(),
        SymbolType::Str => codegen.str_type.into(),
        SymbolType::Bool => codegen.context.bool_type().into(),
        SymbolType::Function(fn_type) => {
            let return_type: BasicTypeEnum = get_llvm_type(&*fn_type.return_type, codegen)
                .try_into()
                .unwrap();

            let mut arg_types = Vec::new();

            for arg in fn_type.args.clone() {
                arg_types.push(get_llvm_type(&arg.arg_type, codegen).try_into().unwrap());
            }

            return_type.fn_type(&arg_types, false).into()
        }
        SymbolType::Array(_) => array_type(codegen).into(),
        SymbolType::Ref(ref_type) => any_type_ptr(get_llvm_type(ref_type, codegen)),
        SymbolType::Map(_, _) => todo!(),
        SymbolType::Struct(s) => {
            let mut symtable_borrow = codegen.symtable.borrow_mut();
            let struct_decl = symtable_borrow.get_struct_decl(s).unwrap();

            let mut field_types = Vec::new();

            for field in struct_decl.fields.values() {
                field_types.push(get_llvm_type(field, codegen).try_into().unwrap());
            }

            codegen.context.struct_type(&field_types, false).into()
        }
        SymbolType::Void => codegen.context.void_type().into(),
        SymbolType::CrocoType => todo!(),
    }
}

pub fn init_default<'ctx>(init_symbol: &LSymbol<'ctx>, codegen: &Codegen<'ctx>) {
    match &init_symbol.symbol_type {
        // stack allocation of a f32
        // TODO: do we really have to give a name ?
        SymbolType::Num => {
            codegen
                .builder
                .build_store(init_symbol.pointer, codegen.context.f32_type().const_zero());
        }

        // stack allocation of a bool
        SymbolType::Bool => {
            codegen.builder.build_store(
                init_symbol.pointer,
                codegen.context.bool_type().const_zero(),
            );
        }
        // TODO: refcount may be needed ?
        // strs and arrays are tougher because they're heap-allocated
        SymbolType::Str | SymbolType::Array(_) => {
            // default initialize all fields
            // the heap ptr is a null ptr
            let heap_ptr = codegen
                .builder
                .build_struct_gep(init_symbol.pointer, 0, "gepheapptr")
                .unwrap();
            let null_ptr = codegen
                .context
                .i8_type()
                .ptr_type(AddressSpace::Generic)
                .const_null();
            codegen.builder.build_store(heap_ptr, null_ptr);

            // both fields defaults to 0
            let len = codegen
                .builder
                .build_struct_gep(init_symbol.pointer, 1, "geplen")
                .unwrap();
            codegen
                .builder
                .build_store(len, codegen.ptr_size.const_int(0, false));

            let max_len = codegen
                .builder
                .build_struct_gep(init_symbol.pointer, 2, "gepmaxlen")
                .unwrap();
            codegen
                .builder
                .build_store(max_len, codegen.ptr_size.const_int(0, false));
        }

        SymbolType::Struct(s) => {
            let mut symtable_borrow = codegen.symtable.borrow_mut();
            let struct_decl = symtable_borrow.get_struct_decl(&s).unwrap();

            for (i, field) in struct_decl.fields.iter().enumerate() {
                let ptr = codegen
                    .builder
                    .build_struct_gep(init_symbol.pointer, i as u32, &field.0)
                    .unwrap();

                let field_symbol = LSymbol {
                    pointer: ptr,
                    symbol_type: field.1.clone(),
                };

                init_default(&field_symbol, codegen);
            }
        }

        // TODO: the checker should catch dangling references like this
        _ => unreachable!(),
    };
}

/// llvm repr of the str type
// https://mapping-high-level-constructs-to-llvm-ir.readthedocs.io/en/latest/appendix-a-how-to-implement-a-string-type-in-llvm/
// {
//     ptr: i8*,
//     len: isize,
//     max_len: isize
// }
// this uses a different size depending on the host's architecture, for performance reasons
pub fn str_type<'ctx>(context: &'ctx Context, ptr_size: IntType) -> StructType<'ctx> {
    let void_ptr = context.i8_type().ptr_type(AddressSpace::Generic).into();
    let isize_type = ptr_size.into();

    context.struct_type(&[void_ptr, isize_type, isize_type], false)
}

/// set the contents of a str
// very inefficient because for large batch of text we're allocating every 16 chars.
// TODO: in the future, pass a void* ptr and a value ?
pub fn set_str_text(str_ptr: PointerValue, text: &str, codegen: &Codegen) {
    let add_char_fn = codegen.module.get_function("_str_add_char").unwrap();

    for el in text.chars() {
        let llvm_char = codegen.context.i8_type().const_int(el as u64, false);
        codegen
            .builder
            .build_call(add_char_fn, &[str_ptr.into(), llvm_char.into()], "");
    }
}

/// a function to add a character to a str
// TODO: less naive impl, with less allocations and growth factor
pub fn register_str_add_char(codegen: &Codegen) -> Result<(), CrocoError> {
    let add_char_ty = codegen.context.void_type().fn_type(
        &[
            codegen.str_type.ptr_type(AddressSpace::Generic).into(),
            codegen.context.i8_type().into(),
        ],
        false,
    );
    let add_char_fn = codegen
        .module
        .add_function("_str_add_char", add_char_ty, None);

    let str_ptr = add_char_fn.get_first_param().unwrap().into_pointer_value();
    let character = add_char_fn.get_last_param().unwrap().into_int_value();

    // entry block of the function
    let entry_block = codegen.context.append_basic_block(add_char_fn, "entry");
    // block if we need to malloc
    let malloc_block = codegen.context.append_basic_block(add_char_fn, "malloc");
    // return block of the function
    let ret_block = codegen.context.append_basic_block(add_char_fn, "end");

    codegen.builder.position_at_end(entry_block);

    let heap_ptr_ptr = codegen
        .builder
        .build_struct_gep(str_ptr, 0, "gepheapptr")
        .unwrap();
    let heap_ptr = codegen
        .builder
        .build_load(heap_ptr_ptr, "loadheapptr")
        .into_pointer_value();

    let len_ptr = codegen
        .builder
        .build_struct_gep(str_ptr, 1, "geplen")
        .unwrap();
    let mut len = codegen
        .builder
        .build_load(len_ptr, "loadlen")
        .into_int_value();

    let max_len_ptr = codegen
        .builder
        .build_struct_gep(str_ptr, 2, "gepmaxlen")
        .unwrap();
    let mut max_len = codegen
        .builder
        .build_load(max_len_ptr, "loadmaxlen")
        .into_int_value();

    // if len and max_len are equal then len+1 will overflow, check this
    let cmp = codegen
        .builder
        .build_int_compare(IntPredicate::EQ, len, max_len, "cmplen");

    codegen
        .builder
        .build_conditional_branch(cmp, malloc_block, ret_block);

    // if we need to allocate more space
    codegen.builder.position_at_end(malloc_block);

    // add to max_len the required space
    let growth_factor = codegen.ptr_size.const_int(16, false);
    max_len = codegen
        .builder
        .build_int_add(max_len, growth_factor, "addgrowth");

    // update our ptr
    codegen.builder.build_store(max_len_ptr, max_len);

    // alloc the new size
    codegen.builder.position_at_end(malloc_block);
    let new_heap_ptr = codegen
        .builder
        .build_array_malloc(codegen.context.i8_type(), max_len, "malloclen")
        .map_err(|_| CrocoError::from_type("heap allocation failed", CrocoErrorKind::Malloc))?;

    // copy heap_ptr into new_heap_ptr
    codegen
        .builder
        .build_memcpy(new_heap_ptr, 8, heap_ptr, 8, len)
        .map_err(|_| CrocoError::from_type("memcpy failed", CrocoErrorKind::Malloc))?;

    // free heap_ptr: if it was a nullptr this shouldn't do anything
    codegen.builder.build_free(heap_ptr);

    // replace heap_ptr by our new_heap_ptr in our string
    codegen.builder.build_store(heap_ptr_ptr, new_heap_ptr);

    // store our new character in the array, we should now have 1 to 16 empty slots
    let new_char_ptr = unsafe { codegen.builder.build_gep(new_heap_ptr, &[len], "gepchar") };
    codegen.builder.build_store(new_char_ptr, character);

    // end the branch
    codegen.builder.build_unconditional_branch(ret_block);
    codegen.builder.position_at_end(ret_block);

    // update our string to our new len
    len = codegen
        .builder
        .build_int_add(len, codegen.ptr_size.const_int(1, false), "addlen");
    codegen.builder.build_store(len_ptr, len);

    codegen.builder.build_return(None);
    Ok(())
}

/// llvm repr of the array type
// defined the same way as a str for now
// {
//     ptr: i8*,
//     len: isize,
//     max_len: isize
// }
#[inline]
pub fn array_type<'ctx>(codegen: &Codegen<'ctx>) -> StructType<'ctx> {
    codegen.str_type
}

/// removes the extension of a file if possible
pub fn strip_ext(file: &str) -> &str {
    Path::new(file)
        .file_stem()
        .unwrap_or_else(|| file.as_ref())
        .to_str()
        .unwrap()
}
