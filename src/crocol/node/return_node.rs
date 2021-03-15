use crate::{
    ast::node::ReturnNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
};
use crate::{crocol::LSymbol, error::CrocoError};

impl CrocolNode for ReturnNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        match self.bottom.crocol(codegen)? {
            LNodeResult::Value(val) => {
                // if we have a struct return, update the pointer and return void
                if let Some(sret_ptr) = codegen.sret_ptr {
                    codegen.builder.build_store(sret_ptr, val.value);
                }
                Ok(LNodeResult::Return(Some(val)))
            }

            LNodeResult::Variable(var) => {
                let val = codegen
                    .builder
                    .build_load(var.value.into_pointer_value(), "loadret");

                // if we have a struct return, update the pointer and return void
                if let Some(sret_ptr) = codegen.sret_ptr {
                    codegen.builder.build_store(sret_ptr, val);
                }
                Ok(LNodeResult::Return(Some(LSymbol {
                    value: val,
                    symbol_type: var.symbol_type,
                })))
            }

            LNodeResult::Void => Ok(LNodeResult::Return(None)),
            _ => Err(CrocoError::invalid_return_value(&self.code_pos)),
        }
    }
}
