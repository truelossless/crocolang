use crate::{
    crocol::{symbol::LSymTable, LCodegen, LSymbol},
    parser::TypedArg,
    symbol::Decl,
    symbol::FunctionDecl,
    symbol_type::SymbolType,
};

use inkwell::{
    types::{BasicType, BasicTypeEnum, StructType},
    AddressSpace,
};
use std::path::Path;

/// get the llvm type corresponding to a SymbolType
pub fn get_llvm_type<'ctx>(
    symbol_type: &SymbolType,
    codegen: &LCodegen<'ctx>,
) -> BasicTypeEnum<'ctx> {
    match symbol_type {
        SymbolType::Num => codegen.context.f32_type().into(),
        SymbolType::Str => codegen.str_type.into(),
        SymbolType::Bool => codegen.context.bool_type().into(),
        SymbolType::Function(_fn_type) => {
            todo!("transform this to a FunctionPointer");
            // https://github.com/TheDan64/inkwell/commit/5a793eba3e0c3a903a0c35da7c61b12790d2c009
            // let return_type: BasicTypeEnum = get_llvm_type(&*fn_type.return_type, codegen);

            // let mut arg_types = Vec::with_capacity(fn_type.args);

            // for arg in fn_type.args.clone() {
            //     arg_types.push(get_llvm_type(&arg.arg_type, codegen).try_into().unwrap());
            // }

            // return_type.fn_type(&arg_types, false).into()
        }
        SymbolType::Array(_) => array_type(codegen).into(),
        SymbolType::Ref(ref_type) => get_llvm_type(ref_type, codegen)
            .ptr_type(AddressSpace::Generic)
            .into(),
        SymbolType::Map(_, _) => todo!(),
        SymbolType::Struct(s) => {
            let struct_decl = codegen.symtable.get_struct_decl(s).unwrap();

            let mut field_types = Vec::with_capacity(struct_decl.fields.len());

            for field in struct_decl.fields.values() {
                field_types.push(get_llvm_type(field, codegen));
            }

            codegen.context.struct_type(&field_types, false).into()
        }
        SymbolType::CrocoType => unreachable!(),
    }
}

pub fn init_default<'ctx>(init_symbol: &LSymbol<'ctx>, codegen: &LCodegen<'ctx>) {
    // we're guarenteed to have a pointer here
    let ptr = init_symbol.value.into_pointer_value();

    match &init_symbol.symbol_type {
        // stack allocation of a f32
        SymbolType::Num => {
            codegen
                .builder
                .build_store(ptr, codegen.context.f32_type().const_zero());
        }

        // stack allocation of a bool
        SymbolType::Bool => {
            codegen
                .builder
                .build_store(ptr, codegen.context.bool_type().const_zero());
        }
        // TODO: refcount may be needed ?
        // strs and arrays are tougher because they're heap-allocated
        SymbolType::Str | SymbolType::Array(_) => {
            // default initialize all fields
            // the heap ptr is a null ptr
            let heap_ptr = codegen
                .builder
                .build_struct_gep(ptr, 0, "gepheapptr")
                .unwrap();
            let null_ptr = codegen
                .context
                .i8_type()
                .ptr_type(AddressSpace::Generic)
                .const_null();
            codegen.builder.build_store(heap_ptr, null_ptr);

            // both fields defaults to 0
            let len = codegen.builder.build_struct_gep(ptr, 1, "geplen").unwrap();
            codegen
                .builder
                .build_store(len, codegen.ptr_size.const_int(0, false));

            let max_len = codegen
                .builder
                .build_struct_gep(ptr, 2, "gepmaxlen")
                .unwrap();
            codegen
                .builder
                .build_store(max_len, codegen.ptr_size.const_int(0, false));
        }

        SymbolType::Struct(s) => {
            let struct_decl = codegen.symtable.get_struct_decl(&s).unwrap();

            for (i, field) in struct_decl.fields.iter().enumerate() {
                let field_ptr = codegen
                    .builder
                    .build_struct_gep(ptr, i as u32, &field.0)
                    .unwrap();

                let field_symbol = LSymbol {
                    value: field_ptr.into(),
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
// pub fn str_type<'ctx>(context: &'ctx Context, ptr_size: IntType) -> StructType<'ctx> {
//     let void_ptr = context.i8_type().ptr_type(AddressSpace::Generic).into();
//     let isize_type = ptr_size.into();

//     context.struct_type(&[void_ptr, isize_type, isize_type], false)
// }

/// llvm repr of the array type
// defined the same way as a str for now
// {
//     ptr: i8*,
//     len: isize,
//     max_len: isize
// }
#[inline]
pub fn array_type<'ctx>(codegen: &LCodegen<'ctx>) -> StructType<'ctx> {
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

/// Inserts all the function definitions from the crocol std
pub fn insert_builtin_functions<'ctx>(symtable: &mut LSymTable<'ctx>) {
    let println_decl = FunctionDecl {
        args: vec![TypedArg {
            arg_name: String::new(),
            arg_type: SymbolType::Str,
        }],
        return_type: None,
    };

    symtable
        .register_decl("println".to_owned(), Decl::FunctionDecl(println_decl))
        .unwrap()
}
