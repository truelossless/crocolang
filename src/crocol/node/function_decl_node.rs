use crate::crocol::{LCodegen, LNodeResult, LSymbol};
use crate::{
    ast::node::FunctionDeclNode,
    crocol::{utils::get_or_define_function, CrocolNode},
};
use crate::{error::CrocoError, symbol_type::SymbolType};
use inkwell::attributes::{Attribute, AttributeLoc};

impl CrocolNode for FunctionDeclNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let fn_decl = codegen
            .symtable
            .get_function_decl(&self.name)
            .unwrap()
            .clone();

        // we're done with the current variables
        codegen.symtable.pop_symbols();

        let function = get_or_define_function(&self.name, &fn_decl, codegen);
        let sret_fn =
            matches!(fn_decl.return_type, Some(SymbolType::Struct(_)) | Some(SymbolType::Str));

        // add the sret tag to the first param if needed
        if sret_fn {
            codegen.sret_ptr = Some(function.get_first_param().unwrap().into_pointer_value());
            let sret_num = Attribute::get_named_enum_kind_id("sret");
            let sret_attr = codegen.context.create_enum_attribute(sret_num, 0);
            function.add_attribute(AttributeLoc::Param(0), sret_attr);
        } else {
            codegen.sret_ptr = None
        }

        // the start of the function
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

        Ok(LNodeResult::Void)
    }
}
