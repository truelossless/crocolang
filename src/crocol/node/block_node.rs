use crate::error::CrocoError;
use crate::{
    ast::node::BlockNode,
    crocol::{LCodegen, LNodeResult},
};
use crate::{ast::BlockScope, crocol::CrocolNode};

impl CrocolNode for BlockNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        match self.scope {
            BlockScope::New | BlockScope::Function => codegen.symtable.add_scope(),
            BlockScope::Keep => (),
        }

        let mut value = LNodeResult::Void;

        for node in &mut self.body {
            value = node.crocol(codegen)?;

            match &value {
                LNodeResult::Return(ret) => {
                    match ret {
                        Some(ret_val) if codegen.sret_ptr.is_none() => {
                            codegen.builder.build_return(Some(&ret_val.value));
                        }
                        _ => {
                            codegen.builder.build_return(None);
                        }
                    }
                    break;
                }
                LNodeResult::Variable(_) | LNodeResult::Value(_) | LNodeResult::Void => (),
                LNodeResult::Break | LNodeResult::Continue => break,
            }
        }

        // return void if there is no return value
        if let LNodeResult::Value(_) | LNodeResult::Variable(_) = value {
            value = LNodeResult::Void;
        };

        // if there is no return the function returns void
        if self.scope == BlockScope::Function {
            match &value {
                LNodeResult::Return(_) => (),
                _ => {
                    codegen.builder.build_return(None);
                }
            }
        }

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New | BlockScope::Function => codegen.symtable.drop_scope(),
            BlockScope::Keep => (),
        }

        Ok(value)
    }
}
