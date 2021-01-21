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

        let mut early_return = false;

        for node in &mut self.body {
            match node.crocol(codegen)? {
                LNodeResult::Return(ret) => {
                    if let Some(ret_val) = ret {
                        codegen.builder.build_return(Some(&ret_val.value));
                    } else {
                        codegen.builder.build_return(None);
                    }
                    early_return = true;
                    break;
                }
                LNodeResult::Variable(_) | LNodeResult::Value(_) | LNodeResult::Void => (),
                _ => unimplemented!(),
            }
        }

        // if there is no early return the function returns void
        if !early_return && self.scope == BlockScope::Function {
            codegen.builder.build_return(None);
        }

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New | BlockScope::Function => codegen.symtable.drop_scope(),
            BlockScope::Keep => (),
        }

        // if there's an early return in this block report it.
        // TODO: less hacky way to do this.
        // maybe have an exit block ? how would it solve this issue ?
        if early_return {
            Ok(LNodeResult::Return(None))
        } else {
            Ok(LNodeResult::Void)
        }
    }
}
