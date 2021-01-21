use crate::symbol_type::SymbolType;
use crate::{ast::node::IfNode, crocol::CrocolNode, error::CrocoError};
use {
    crate::crocol::{LCodegen, LNodeResult},
    inkwell::IntPredicate,
};

impl CrocolNode for IfNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let true_value = codegen.context.bool_type().const_int(1, false);
        let before_if_block = codegen.builder.get_insert_block().unwrap();

        // We need to iterate backwards or the condition blocks wouldn't be created.
        // this results in llvm ir code with conditions blocks which can be confusing,
        // but it's better than other solutions tried which involved `Vec` allocation.
        // here is the intended result for reference:
        //
        // entry:
        //   ...
        //   br if1
        //
        // then2:
        //   ...
        //   br endif
        //
        // if2:
        //   %cond = cmp ...
        //   br %cond then2 endif
        //
        // then1:
        //   ...
        //   br endif
        //
        // if1:
        //   %cond = cmp ...
        //   br %cond then1 if2

        // the block closing the condition
        let endif_block = codegen
            .context
            .append_basic_block(codegen.current_fn.unwrap(), "endif");

        // the next if block.
        // Since we iterate backwards it is at the start of either the else block, or the endif block
        let mut next_block = if self.conditions.len() != self.bodies.len() {
            let else_block = codegen
                .context
                .append_basic_block(codegen.current_fn.unwrap(), "else");

            // populate the else block
            codegen.builder.position_at_end(else_block);
            self.bodies.last_mut().unwrap().crocol(codegen)?;
            codegen.builder.build_unconditional_branch(endif_block);
            codegen.builder.position_at_end(before_if_block);

            else_block
        } else {
            endif_block
        };

        for (condition, body) in self.conditions.iter_mut().zip(self.bodies.iter_mut()).rev() {
            let if_block = codegen
                .context
                .append_basic_block(codegen.current_fn.unwrap(), "if");
            let then_block = codegen
                .context
                .append_basic_block(codegen.current_fn.unwrap(), "then");

            // populate the new then block
            codegen.builder.position_at_end(then_block);

            // llvm doesn't like when two terminators are in the same block.
            // if we have a early return, we don't want to build a branch.
            match body.crocol(codegen)? {
                LNodeResult::Return(None) => (),
                _ => {
                    codegen.builder.build_unconditional_branch(endif_block);
                }
            }

            // populate the new if block
            codegen.builder.position_at_end(if_block);

            let cond_ok = condition
                .crocol(codegen)?
                .into_symbol(codegen, &self.code_pos)?;

            match cond_ok.symbol_type {
                SymbolType::Bool => (),
                _ => return Err(CrocoError::condition_not_bool_error(&self.code_pos)),
            }

            let cmp = codegen.builder.build_int_compare(
                IntPredicate::EQ,
                cond_ok.value.into_int_value(),
                true_value,
                "cmpif",
            );
            codegen
                .builder
                .build_conditional_branch(cmp, then_block, next_block);

            next_block = if_block;
        }

        // link the previous block to our first if block
        codegen.builder.position_at_end(before_if_block);
        codegen.builder.build_unconditional_branch(next_block);

        // move the endif block at the end of all our created blocks
        endif_block
            .move_after(codegen.current_fn.unwrap().get_last_basic_block().unwrap())
            .unwrap();
        codegen.builder.position_at_end(endif_block);

        Ok(LNodeResult::Void)
    }
}
