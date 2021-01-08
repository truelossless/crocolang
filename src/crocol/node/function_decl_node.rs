use crate::error::CrocoError;
use crate::symbol::Decl;
use crate::{ast::node::FunctionDeclNode, crocol::CrocolNode};

use {
    crate::crocol::{utils::get_llvm_type, LCodegen, LNodeResult, LSymbol},
    inkwell::types::BasicType,
};

impl CrocolNode for FunctionDeclNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let fn_decl = self.fn_decl.take().unwrap();

        // convert the arguments to llvm
        let llvm_args: Vec<_> = fn_decl
            .args
            .iter()
            .map(|x| get_llvm_type(&x.arg_type, codegen))
            .collect();

        let fn_ty = if let Some(return_type) = &fn_decl.return_type {
            get_llvm_type(return_type, codegen).fn_type(&llvm_args, false)
        } else {
            codegen.context.void_type().fn_type(&llvm_args, false)
        };

        codegen.current_fn = Some(codegen.module.add_function(&self.name, fn_ty, None));
        let entry = codegen
            .context
            .append_basic_block(codegen.current_fn.unwrap(), "entry");
        codegen.builder.position_at_end(entry);

        // inject the function arguments in the body
        for (arg, llvm_ty) in fn_decl.args.iter().zip(llvm_args) {
            let ptr = codegen.create_block_alloca(llvm_ty, "allocaarg");
            let symbol = LSymbol {
                symbol_type: arg.arg_type.clone(),
                value: ptr.into(),
            };
            codegen
                .symtable
                .insert_symbol(&arg.arg_name, symbol)
                .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        }

        self.fn_body.as_mut().unwrap().crocol(codegen)?;

        codegen
            .symtable
            .register_decl(self.name.clone(), Decl::FunctionDecl(fn_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, &e))?;

        Ok(LNodeResult::Void)
    }
}
