use crate::error::CrocoError;
use crate::{
    ast::node::ReturnNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
};

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
                    Ok(LNodeResult::Return(None))
                } else {
                    Ok(LNodeResult::Return(Some(val)))
                }
            }

            LNodeResult::Variable(var) => {
                let val = codegen
                    .builder
                    .build_load(var.value.into_pointer_value(), "loadsret");
                // if we have a struct return, update the pointer and return void
                if let Some(sret_ptr) = codegen.sret_ptr {
                    codegen.builder.build_store(sret_ptr, val);
                    Ok(LNodeResult::Return(None))
                } else {
                    Ok(LNodeResult::Return(Some(var)))
                }
            }

            LNodeResult::Void => Ok(LNodeResult::Return(None)),
            _ => Err(CrocoError::invalid_return_value(&self.code_pos)),
        }
    }
}
