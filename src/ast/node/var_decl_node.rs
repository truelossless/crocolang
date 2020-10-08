use crate::error::CrocoError;
use crate::{
    ast::{AstNode, INodeResult},
    symbol_type::SymbolType,
    token::CodePos,
};

#[cfg(feature = "crocol")]
use {
    crate::{
        crocol::LSymbol,
        crocol::{utils::get_llvm_type, Codegen, LNodeResult},
    },
    inkwell::{types::BasicTypeEnum, values::BasicValueEnum},
    std::convert::TryInto,
};

#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::get_symbol_type, symbol::ISymTable, utils::init_default};

use std::cell::RefCell;
use std::rc::Rc;

/// a node to declare a new variable (declared variable are initialized by default)
#[derive(Clone)]
pub struct VarDeclNode {
    // the var_name
    left: String,
    // the variable Assignement (None for a default assignment)
    right: Option<Box<dyn AstNode>>,
    // the type of the variable
    var_type: Option<SymbolType>,
    code_pos: CodePos,
}

impl VarDeclNode {
    pub fn new(
        var_name: String,
        expr: Option<Box<dyn AstNode>>,
        var_type: Option<SymbolType>,
        code_pos: CodePos,
    ) -> Self {
        VarDeclNode {
            left: var_name,
            right: expr,
            var_type,
            code_pos,
        }
    }
}

impl AstNode for VarDeclNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let value = match &mut self.right {
            // there is a node
            Some(node) => {
                let var_value = node.crocoi(symtable)?.into_symbol(&self.code_pos)?;

                // type differs from annotation
                if let Some(var_type) = &self.var_type {
                    if !get_symbol_type(&var_value).eq(var_type) {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            &format!(
                            "variable {} has been explicitely given a type but is declared with another one",
                            &self.left),
                        ));
                    }
                }

                var_value
            }

            // no node, use the defaut value
            None => match &self.var_type {
                None => {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        &format!("cannot infer the type of the variable {}", self.left),
                    ))
                }

                Some(var_type) => init_default(var_type, symtable, &self.code_pos)?,
            },
        };

        symtable
            .insert_symbol(&self.left, Rc::new(RefCell::new(value)))
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(INodeResult::Void)
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        // TODO: remove once checker works
        self.var_type = Some(SymbolType::Str);

        let ty: BasicTypeEnum = get_llvm_type(&self.var_type.as_ref().unwrap(), codegen)
            .try_into()
            .unwrap();
        let alloca = codegen.create_entry_block_alloca(ty, &self.left);

        let symbol = LSymbol {
            value: alloca.into(),
            symbol_type: self.var_type.as_ref().unwrap().clone(),
        };

        match &mut self.right {
            Some(node) => {
                let right_val: BasicValueEnum =
                    node.crocol(codegen)?.into_value(&self.code_pos)?.value;
                codegen.builder.build_store(alloca, right_val);
            }

            None => {
                // the checker should ensure that the type is valid
                // we can just default assign
                crate::crocol::utils::init_default(&symbol, codegen);
            }
        }

        codegen.symtable.insert_symbol(&self.left, symbol).unwrap();

        Ok(LNodeResult::Void)
    }
}
