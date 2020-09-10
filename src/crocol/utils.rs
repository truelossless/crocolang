use crate::{
    crocol::{Codegen, LSymbol},
    symbol_type::SymbolType,
};
use inkwell::{
    types::{AnyTypeEnum, BasicType, BasicTypeEnum, StructType},
    AddressSpace,
};
use std::convert::TryInto;

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
pub fn get_llvm_type<'ctx>(symbol_type: &SymbolType, codegen: &'ctx Codegen) -> AnyTypeEnum<'ctx> {
    match symbol_type {
        SymbolType::Num => codegen.context.f32_type().into(),
        SymbolType::Str => str_type(codegen).into(),
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

pub fn _init_default<'ctx>(init_symbol: &LSymbol<'ctx>, codegen: &Codegen<'ctx>) {

    match &init_symbol.symbol_type {
        // stack allocation of a f32
        // TODO: do we really have to give a name ?
        SymbolType::Num => {
            codegen
                .builder
                .build_store(init_symbol.pointer, codegen.context.f64_type().const_zero());
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
                .build_struct_gep(init_symbol.pointer, 0, "heapptr")
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
                .build_struct_gep(init_symbol.pointer, 1, "len")
                .unwrap();
            codegen
                .builder
                .build_store(len, codegen.ptr_size.const_int(0, false));

            let max_len = codegen
                .builder
                .build_struct_gep(init_symbol.pointer, 2, "max_len")
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

                _init_default(&field_symbol, codegen);
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
pub fn str_type<'ctx>(codegen: &'ctx Codegen) -> StructType<'ctx> {
    let void_ptr = codegen
        .context
        .i8_type()
        .ptr_type(AddressSpace::Generic)
        .into();
    let isize_type = codegen.ptr_size.into();

    codegen
        .context
        .struct_type(&[void_ptr, isize_type, isize_type], false)
}

/// llvm repr of the array type
// defined the same way as a str for now
// {
//     ptr: i8*,
//     len: isize,
//     max_len: isize
// }
#[inline]
pub fn array_type<'ctx>(codegen: &'ctx Codegen) -> StructType<'ctx> {
    str_type(codegen)
}
