use crate::crocol::{LCodegen, LNodeResult, LSymbol};
use crate::error::CrocoError;
use crate::symbol_type::SymbolType;
use crate::{ast::node::*, crocol::CrocolNode};

impl CrocolNode for FunctionCallNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let mut visited_args = Vec::with_capacity(self.args.len());
        for arg in &mut self.args {
            let mut value = arg.crocol(codegen)?.into_symbol(codegen, &self.code_pos)?;

            // when passing structs to a function it's better to pass pointers, and then
            // if we want to pass the struct by value we can just stack allocate the fields in the function body.
            // this prevents quirks when passing large values, and it is also used by clang.
            value = match value.symbol_type {
                SymbolType::Struct(_) => {
                    let alloca = codegen.create_block_alloca(value.value.get_type(), "tmpstruct");
                    codegen.builder.build_store(alloca, value.value);

                    LSymbol {
                        value: alloca.into(),
                        // here the type differs from the value, so we can check if we pass by value and have a pointer
                        // when value is a PointerValue and symbol_type is Struct
                        symbol_type: value.symbol_type,
                    }
                }
                _ => value,
            };

            visited_args.push(value.value);
        }

        let fn_decl = codegen
            .symtable
            .get_function_decl(&self.fn_name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        let function = codegen.module.get_function(&self.fn_name).unwrap();

        let res = codegen
            .builder
            .build_call(function, &visited_args, "callfn");

        if let Some(ret) = &fn_decl.return_type {
            let return_symbol = LSymbol {
                value: res.try_as_basic_value().left().unwrap(),
                symbol_type: ret.clone(),
            };
            Ok(LNodeResult::Value(return_symbol))
        } else {
            Ok(LNodeResult::Void)
        }
    }
}
