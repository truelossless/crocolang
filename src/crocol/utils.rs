use crate::{
    crocol::{symbol::LSymTable, LCodegen, LSymbol},
    parser::TypedArg,
    symbol::Decl,
    symbol::{FunctionDecl, StructDecl},
    symbol_type::SymbolType,
};

use inkwell::{
    types::{BasicType, BasicTypeEnum, StructType},
    values::FunctionValue,
    AddressSpace,
};
use std::{path::Path, vec};

/// Auto dereferences as many times as needed  
pub fn auto_deref<'ctx>(mut symbol: LSymbol<'ctx>, codegen: &LCodegen<'ctx>) -> LSymbol<'ctx> {
    while let SymbolType::Ref(r) = symbol.symbol_type {
        symbol = LSymbol {
            value: codegen
                .builder
                .build_load(symbol.value.into_pointer_value(), "autoderef"),
            symbol_type: *r,
        };
    }

    symbol
}

/// Gets the llvm type corresponding to a SymbolType
pub fn get_llvm_type<'ctx>(
    symbol_type: &SymbolType,
    codegen: &LCodegen<'ctx>,
) -> BasicTypeEnum<'ctx> {
    match symbol_type {
        SymbolType::Num => codegen.context.i32_type().into(),
        SymbolType::Fnum => codegen.context.f32_type().into(),
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
        SymbolType::Array(_) => codegen.str_type.into(),
        SymbolType::Ref(ref_type) => get_llvm_type(ref_type, codegen)
            .ptr_type(AddressSpace::Generic)
            .into(),
        SymbolType::Map(_, _) => todo!(),
        SymbolType::Struct(s) => {
            let struct_decl = codegen.symtable.get_struct_decl(s).unwrap();
            get_or_define_struct(s, struct_decl, codegen).into()
        }
        SymbolType::CrocoType => unreachable!(),
    }
}

/// Default initializes a symbol
pub fn init_default<'ctx>(init_symbol: &LSymbol<'ctx>, codegen: &LCodegen<'ctx>) {
    // we're guarenteed to have a pointer here
    let ptr = init_symbol.value.into_pointer_value();

    match &init_symbol.symbol_type {
        // stack allocation of an i32
        SymbolType::Num => {
            codegen
                .builder
                .build_store(ptr, codegen.context.i32_type().const_zero());
        }

        // stack allocation of a f32
        SymbolType::Fnum => {
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

        _ => unimplemented!(),
    };
}

/// Removes the extension of a file if possible
pub fn strip_ext(file: &str) -> &str {
    Path::new(file)
        .file_stem()
        .unwrap_or_else(|| file.as_ref())
        .to_str()
        .unwrap()
}

/// Returns the inkwell function if it exists, or create one according to the croco function definition
pub fn get_or_define_function<'ctx>(
    fn_name: &str,
    fn_decl: &FunctionDecl,
    codegen: &LCodegen<'ctx>,
) -> FunctionValue<'ctx> {
    codegen.module.get_function(fn_name).unwrap_or_else(|| {
        // create a new llvm function

        // convert the arguments to llvm
        // to comply with the "C ABI", fn(Struct a) is changed to fn(&Struct a)
        let mut llvm_args = Vec::with_capacity(fn_decl.args.len());
        for arg in fn_decl.args.iter() {
            let llvm_arg = match arg.arg_type {
                SymbolType::Str | SymbolType::Struct(_) => get_llvm_type(&arg.arg_type, codegen)
                    .ptr_type(AddressSpace::Generic)
                    .into(),
                SymbolType::Num | SymbolType::Fnum | SymbolType::Bool | SymbolType::Ref(_) => {
                    get_llvm_type(&arg.arg_type, codegen)
                }
                _ => unimplemented!(),
            };

            llvm_args.push(llvm_arg);
        }

        // if the return type is a struct, pass as the first argument a pointer to this struct which
        // will contain the result of the function.
        let fn_ty = match &fn_decl.return_type {
            Some(SymbolType::Str) | Some(SymbolType::Struct(_)) => {
                llvm_args.insert(
                    0,
                    get_llvm_type(&fn_decl.return_type.as_ref().unwrap(), codegen)
                        .ptr_type(AddressSpace::Generic)
                        .into(),
                );
                codegen.context.void_type().fn_type(&llvm_args, false)
            }

            Some(SymbolType::Bool) | Some(SymbolType::Fnum) | Some(SymbolType::Num) => {
                let ret_ty = get_llvm_type(&fn_decl.return_type.as_ref().unwrap(), codegen);
                ret_ty.fn_type(&llvm_args, false)
            }

            None => codegen.context.void_type().fn_type(&llvm_args, false),

            _ => unimplemented!(),
        };

        codegen.module.add_function(fn_name, fn_ty, None)
    })
}

pub fn get_or_define_struct<'ctx>(
    struct_name: &str,
    struct_decl: &StructDecl,
    codegen: &LCodegen<'ctx>,
) -> StructType<'ctx> {
    codegen
        .module
        .get_struct_type(struct_name)
        .unwrap_or_else(|| {
            let opaque = codegen.context.opaque_struct_type(&struct_name);

            let llvm_fields: Vec<_> = struct_decl
                .fields
                .values()
                .map(|x| get_llvm_type(x, codegen))
                .collect();

            opaque.set_body(&llvm_fields, false);

            opaque
        })
}
/// Inserts all the function definitions from the crocol std
pub fn insert_builtin_functions<'ctx>(symtable: &mut LSymTable<'ctx>) {
    let assert_decl = FunctionDecl {
        args: vec![TypedArg {
            arg_name: String::new(),
            arg_type: SymbolType::Bool,
        }],
        return_type: None,
    };

    symtable
        .register_decl("assert".to_owned(), Decl::FunctionDecl(assert_decl))
        .unwrap();

    let print_decl = FunctionDecl {
        args: vec![TypedArg {
            arg_name: String::new(),
            arg_type: SymbolType::Str,
        }],
        return_type: None,
    };

    symtable
        .register_decl("eprint".to_owned(), Decl::FunctionDecl(print_decl.clone()))
        .unwrap();

    symtable
        .register_decl(
            "eprintln".to_owned(),
            Decl::FunctionDecl(print_decl.clone()),
        )
        .unwrap();

    symtable
        .register_decl("print".to_owned(), Decl::FunctionDecl(print_decl.clone()))
        .unwrap();

    symtable
        .register_decl("println".to_owned(), Decl::FunctionDecl(print_decl))
        .unwrap()
}
