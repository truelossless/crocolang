use crate::ast::{AstNode, INodeResult};
use crate::error::CrocoError;
use crate::symbol::{get_symbol_type, SymTable};
use crate::{
    crocoi::{symbol::SymbolContent, utils::init_default, ISymbol},
    crocol::{utils::get_llvm_type, Codegen, LNodeResult},
    symbol_type::{type_eq, SymbolType},
    token::{CodePos, LiteralEnum::*},
};

use inkwell::{types::BasicTypeEnum, values::BasicValueEnum};
use std::cell::RefCell;
use std::{convert::TryInto, rc::Rc};

/// a node to declare a new variable (declared variable are initialized by default)
#[derive(Clone)]
pub struct VarDeclNode {
    // the var_name
    left: String,
    // the variable Assignement (None for a default assignment)
    right: Option<Box<dyn AstNode>>,
    // the type of the variable
    var_type: SymbolType,
    code_pos: CodePos,
}

impl VarDeclNode {
    pub fn new(
        var_name: String,
        expr: Option<Box<dyn AstNode>>,
        var_type: SymbolType,
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
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let value = match &mut self.right {
            // there is a node
            Some(node) => {
                let var_value = node.crocoi(symtable)?.into_symbol(&self.code_pos)?;
                let var_value_borrow = var_value.borrow();

                // type differs from annotation
                if !self.var_type.is_void()
                    && !type_eq(&get_symbol_type(&*var_value_borrow), &self.var_type)
                {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        &format!(
                        "variable {} has been explicitely given a type but is declared with another one",
                        &self.left),
                    ));
                }

                // no annotation at all
                if var_value_borrow.is_void() && self.var_type.is_void() {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        &format!("trying to assign a void expression to {}", self.left),
                    ));
                }

                drop(var_value_borrow);
                var_value
            }

            // no node, use the defaut value
            None => {
                if self.var_type.is_void() {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        &format!("cannot infer the type of the variable {}", self.left),
                    ));
                }

                Rc::new(RefCell::new(init_default(
                    &self.var_type,
                    symtable,
                    &self.code_pos,
                )?))
            }
        };

        symtable
            .insert_symbol(&self.left, value)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            Void,
        )))
    }

    fn crocol<'ctx>(&mut self, codegen: &Codegen<'ctx>) -> Result<LNodeResult<'ctx>, CrocoError> {
        // TODO: remove once checker works
        self.var_type = SymbolType::Num;

        let ty: BasicTypeEnum = get_llvm_type(&self.var_type, codegen).try_into().unwrap();
        let alloca = codegen.builder.build_alloca(ty, &self.left);

        // let symbol = LSymbol {
        //     pointer: alloca,
        //     symbol_type: self.var_type.clone(),
        // };

        match &mut self.right {
            Some(node) => {
                let right_val: BasicValueEnum =
                    node.crocol(codegen)?.into_symbol().try_into().unwrap();
                codegen.builder.build_store(alloca, right_val);
            }

            None => {
                // the checker should ensure that the type is valid
                // we can just default assign
                // lifetime mismatch here !!!! >:(
                // crocol::utils::init_default(&symbol, codegen);
            }
        }

        // let symtable_borrow = codegen.symtable.borrow_mut();
        // lifetime mismatch here !!!! >:(
        // symtable_borrow.insert_symbol(&self.left, symbol).unwrap();

        Ok(LNodeResult::Void)
    }

    fn prepend_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }

    fn add_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }

    fn get_type(&self) -> crate::ast::AstNodeType {
        unimplemented!();
    }
}
