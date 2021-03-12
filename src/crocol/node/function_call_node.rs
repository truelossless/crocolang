use crate::crocol::{
    utils::{auto_deref, get_or_define_function, get_or_define_struct},
    LCodegen, LNodeResult, LSymbol,
};
use crate::error::CrocoError;
use crate::symbol_type::SymbolType;
use crate::{ast::node::*, crocol::CrocolNode};

impl CrocolNode for FunctionCallNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let mut visited_args = Vec::with_capacity(self.args.len());
        let fn_name;
        // if we're dealing with a method, inject self as the first argument
        if let Some(method_self) = self.method.as_mut() {
            let mut method_symbol = method_self
                .crocol(codegen)?
                .into_pointer(codegen, &self.code_pos)?;

            method_symbol = auto_deref(method_symbol, codegen);

            fn_name = match method_symbol.symbol_type {
                SymbolType::Struct(struct_name) => format!("_{}_{}", struct_name, self.fn_name),
                SymbolType::Str => format!("_str_{}", &self.fn_name),
                SymbolType::Fnum => format!("_num_{}", &self.fn_name),
                SymbolType::Bool => format!("_bool_{}", &self.fn_name),
                _ => unimplemented!(),
            };

            visited_args.push(method_symbol.value);
        } else {
            fn_name = self.fn_name.clone();
        };

        let fn_decl = codegen
            .symtable
            .get_function_decl(&fn_name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .clone();

        for (i, (arg, arg_decl)) in self.args.iter_mut().zip(&fn_decl.args).enumerate() {
            let mut value = arg.crocol(codegen)?.into_symbol(codegen, &self.code_pos)?;

            if value.symbol_type != arg_decl.arg_type {
                return Err(CrocoError::parameter_error(
                    &self.code_pos,
                    i,
                    self.method.is_some(),
                ));
            }

            // To comply with the C ABI we pass struct values as pointers
            // It gets memcpy'ied later in the callee.
            value = match value.symbol_type {
                SymbolType::Struct(_) | SymbolType::Str => {
                    let alloca = codegen.create_block_alloca(value.value.get_type(), "tmpstruct");
                    codegen.builder.build_store(alloca, value.value);

                    LSymbol {
                        value: alloca.into(),
                        symbol_type: value.symbol_type,
                    }
                }

                SymbolType::Bool | SymbolType::Fnum | SymbolType::Num => value,
                _ => unimplemented!(),
            };

            visited_args.push(value.value);
        }

        if visited_args.len() != fn_decl.args.len() {
            return Err(CrocoError::mismatched_number_of_arguments_error(
                &self.code_pos,
                fn_decl.args.len(),
                self.args.len(),
            ));
        }

        let function = get_or_define_function(&fn_name, &fn_decl, codegen);

        // to conform to the "C ABI",
        // we change `Struct fn()` to `void fn(Struct*)`
        // TODO: don't do it for small structs
        let maybe_ret_alloca = match fn_decl.return_type {
            Some(SymbolType::Str) | Some(SymbolType::Struct(_)) => {
                let ty = if let SymbolType::Struct(struct_name) =
                    fn_decl.return_type.as_ref().unwrap()
                {
                    let struct_ty = codegen
                        .symtable
                        .get_struct_decl(&struct_name)
                        .map_err(|e| CrocoError::new(&self.code_pos, e))?;
                    get_or_define_struct(&struct_name, struct_ty, codegen)
                } else {
                    codegen.str_type
                };

                let alloca = codegen.create_block_alloca(ty.into(), "sret");
                // insert a pointer to the return value in the first argument
                visited_args.insert(0, alloca.into());
                Some(alloca)
            }
            None | Some(SymbolType::Bool) | Some(SymbolType::Fnum) | Some(SymbolType::Num) => None,
            _ => unimplemented!(),
        };

        let res = codegen
            .builder
            .build_call(function, &visited_args, "callfn");

        // the function should return something
        if let Some(ret) = fn_decl.return_type {
            // sret function result
            let return_symbol = if let Some(alloca) = maybe_ret_alloca {
                // load back the value from the alloca
                // it should have been mutated by the callee
                let value = codegen.builder.build_load(alloca, "loadsret");

                LSymbol {
                    value,
                    symbol_type: ret,
                }

            // classic function result
            } else {
                LSymbol {
                    value: res.try_as_basic_value().left().unwrap(),
                    symbol_type: ret,
                }
            };

            Ok(LNodeResult::Value(return_symbol))
        } else {
            Ok(LNodeResult::Void)
        }
    }
}
