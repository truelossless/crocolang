use crate::{
    ast::node::WhileNode,
    crocol::{CrocolNode, LCodegen, LNodeResult},
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for WhileNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let current_fn = codegen.current_fn.unwrap();
        let cond_block = codegen.context.append_basic_block(current_fn, "ifwhile");
        let while_block = codegen.context.append_basic_block(current_fn, "while");
        let end_block = codegen.context.append_basic_block(current_fn, "endwhile");

        // set in the codegen the current loop
        codegen.current_loop_block = Some(cond_block);
        codegen.current_loop_end_block = Some(end_block);

        // build the condition block
        codegen.builder.build_unconditional_branch(cond_block);
        codegen.builder.position_at_end(cond_block);

        let cond_ok = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        match cond_ok.symbol_type {
            SymbolType::Bool => (),
            _ => return Err(CrocoError::condition_not_bool_error(&self.code_pos)),
        }

        codegen.builder.build_conditional_branch(
            cond_ok.value.into_int_value(),
            while_block,
            end_block,
        );

        // build the while body
        codegen.builder.position_at_end(while_block);
        let value = self.right.as_mut().unwrap().crocol(codegen)?;
        match value {
            // as in the if node, do not put two terminators next to each other.
            LNodeResult::Return(_) | LNodeResult::Continue | LNodeResult::Break => (),
            _ => {
                codegen.builder.build_unconditional_branch(cond_block);
            }
        }

        codegen.builder.position_at_end(end_block);
        Ok(LNodeResult::Void)
    }
}
