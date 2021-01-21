use crate::crocol::{utils::get_or_define_function, LCodegen, LNodeResult, LSymbol};
use crate::error::CrocoError;
use crate::symbol_type::SymbolType;
use crate::{ast::node::*, crocol::CrocolNode};

impl CrocolNode for FunctionCallNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let fn_decl = codegen
            .symtable
            .get_function_decl(&self.fn_name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .clone();

        let mut visited_args = Vec::with_capacity(self.args.len());
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
                        // here the type differs from the value, so we can check if we pass by value and have a pointer
                        // when value is a PointerValue and symbol_type is Struct
                        symbol_type: value.symbol_type,
                    }
                }

                SymbolType::Bool | SymbolType::Num => value,
                _ => unimplemented!(),
            };

            visited_args.push(value.value);
        }

        if self.args.len() != fn_decl.args.len() {
            return Err(CrocoError::mismatched_number_of_arguments_error(
                &self.code_pos,
                fn_decl.args.len(),
                self.args.len(),
            ));
        }

        let function = get_or_define_function(&self.fn_name, &fn_decl, codegen);

        // to conform to the "C ABI",
        // we change `Struct fn()` to `void fn(Struct*)`
        // TODO: don't do it for small structs
        let maybe_ret_alloca = match fn_decl.return_type {
            Some(SymbolType::Str) => {
                let str_ty = codegen.str_type;
                let alloca = codegen.create_block_alloca(str_ty.into(), "sretstr");
                // insert a pointer to the return value in the first argument
                visited_args.insert(0, alloca.into());
                Some(alloca)
            }
            None | Some(SymbolType::Bool) | Some(SymbolType::Num) => None,
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
