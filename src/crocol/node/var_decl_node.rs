use crate::{ast::node::VarDeclNode, symbol_type::SymbolType};
use crate::{crocol::CrocolNode, error::CrocoError};

use {
    crate::{
        crocol::LSymbol,
        crocol::{utils::get_llvm_type, LCodegen, LNodeResult},
    },
    inkwell::{types::BasicType, AddressSpace},
};

impl CrocolNode for VarDeclNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let symbol: LSymbol;

        match &mut self.right {
            Some(node) => {
                let right = node.crocol(codegen)?.into_symbol(codegen, &self.code_pos)?;

                if let Some(var_type) = &self.var_type {
                    if !var_type.eq(&right.symbol_type) {
                        return Err(CrocoError::type_annotation_error(
                            &self.code_pos,
                            &self.left,
                        ));
                    }
                }

                let mut llvm_type = get_llvm_type(&right.symbol_type, codegen);

                // str is a bit special and has already been stack allocated
                llvm_type = match &right.symbol_type {
                    SymbolType::Str => llvm_type.ptr_type(AddressSpace::Generic).into(),
                    _ => llvm_type,
                };

                let alloca = codegen.create_block_alloca(llvm_type, &self.left);

                codegen.builder.build_store(alloca, right.value);

                symbol = LSymbol {
                    value: alloca.into(),
                    symbol_type: right.symbol_type,
                };
            }

            None => match &self.var_type {
                None => return Err(CrocoError::infer_error(&self.code_pos, &self.left)),

                Some(var_type) => {
                    let llvm_type = get_llvm_type(&var_type, codegen);
                    let alloca = codegen.create_block_alloca(llvm_type, &self.left);

                    symbol = LSymbol {
                        value: alloca.into(),
                        symbol_type: var_type.clone(),
                    };

                    crate::crocol::utils::init_default(&symbol, codegen);
                }
            },
        }

        codegen
            .symtable
            .insert_symbol(&self.left, symbol)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        Ok(LNodeResult::Void)
    }
}
