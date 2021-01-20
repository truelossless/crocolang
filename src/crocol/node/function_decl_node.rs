use inkwell::{
    attributes::{Attribute, AttributeLoc},
    AddressSpace,
};

use crate::symbol::Decl;
use crate::{ast::node::FunctionDeclNode, crocol::CrocolNode};
use crate::{error::CrocoError, symbol_type::SymbolType};

use {
    crate::crocol::{utils::get_llvm_type, LCodegen, LNodeResult, LSymbol},
    inkwell::types::BasicType,
};

impl CrocolNode for FunctionDeclNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let fn_decl = self.fn_decl.take().unwrap();

        // we're done with the current variables
        codegen.symtable.pop_symbols();

        // convert the arguments to llvm
        // to comply with the "C ABI", fn(Struct a) is changed to fn(&Struct a)
        let mut llvm_args = Vec::with_capacity(fn_decl.args.len());
        for arg in fn_decl.args.iter() {
            let llvm_arg = match arg.arg_type {
                SymbolType::Str => codegen.str_type.ptr_type(AddressSpace::Generic).into(),
                SymbolType::Num | SymbolType::Bool => get_llvm_type(&arg.arg_type, codegen),
                _ => unimplemented!(),
            };

            llvm_args.push(llvm_arg);
        }

        // if the return type is a struct, pass as the first argument a pointer to this struct which
        // will contain the result of the function.
        let mut sret_fn = false;

        let fn_ty = match &fn_decl.return_type {
            Some(SymbolType::Str) => {
                sret_fn = true;
                llvm_args.insert(0, codegen.str_type.ptr_type(AddressSpace::Generic).into());
                codegen.context.void_type().fn_type(&llvm_args, false)
            }

            Some(SymbolType::Bool) | Some(SymbolType::Num) => {
                let ret_ty = get_llvm_type(&fn_decl.return_type.as_ref().unwrap(), codegen);
                ret_ty.fn_type(&llvm_args, false)
            }

            None => codegen.context.void_type().fn_type(&llvm_args, false),

            _ => unimplemented!(),
        };

        let function = codegen.module.add_function(&self.name, fn_ty, None);

        // add the sret tag to the first param if needed
        if sret_fn {
            codegen.sret_ptr = Some(function.get_first_param().unwrap().into_pointer_value());
            let sret_num = Attribute::get_named_enum_kind_id("sret");
            let sret_attr = codegen.context.create_enum_attribute(sret_num, 0);
            function.add_attribute(AttributeLoc::Param(0), sret_attr);
        } else {
            codegen.sret_ptr = None
        }

        let entry = codegen.context.append_basic_block(function, "entry");
        codegen.builder.position_at_end(entry);

        // in case of a function returning a struct, skip the first param which is the return value
        let args_iter = if sret_fn {
            fn_decl.args.iter().zip(function.get_param_iter().skip(1))
        } else {
            // skip 0 so both iters have the same type
            fn_decl.args.iter().zip(function.get_param_iter().skip(0))
        };

        codegen.current_fn = Some(function);

        // inject the function arguments in the body
        for (arg, param_value) in args_iter {
            // to comply with the "C ABI", fn(Struct a) is changed to fn(&Struct a)
            // this means we need to add a memcpy in the function body.
            let abi_ptr = match &arg.arg_type {
                SymbolType::Str => {
                    let copy_alloca =
                        codegen.create_block_alloca(codegen.str_type.into(), "valstr");

                    codegen
                        .builder
                        .build_memcpy(
                            copy_alloca,
                            8,
                            param_value.into_pointer_value(),
                            8,
                            codegen.str_type.size_of().unwrap(),
                        )
                        .unwrap();
                    copy_alloca
                }
                SymbolType::Bool | SymbolType::Num => {
                    let param_ptr = codegen.create_block_alloca(param_value.get_type(), "param");
                    codegen.builder.build_store(param_ptr, param_value);
                    param_ptr
                }
                _ => unimplemented!(),
            };

            let symbol = LSymbol {
                symbol_type: arg.arg_type.clone(),
                value: abi_ptr.into(),
            };
            codegen
                .symtable
                .insert_symbol(&arg.arg_name, symbol)
                .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        }

        self.fn_body.as_mut().unwrap().crocol(codegen)?;

        codegen
            .symtable
            .register_decl(self.name.clone(), Decl::FunctionDecl(fn_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, &e))?;

        Ok(LNodeResult::Void)
    }
}
