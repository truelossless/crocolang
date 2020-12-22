use crate::{ast::node::VarDeclNode, crocoi::INodeResult};
use crate::{crocoi::CrocoiNode, error::CrocoError};

use crate::crocoi::{symbol::get_symbol_type, symbol::ICodegen, utils::init_default};
use std::cell::RefCell;
use std::rc::Rc;

impl CrocoiNode for VarDeclNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let value = match &mut self.right {
            // there is a node
            Some(node) => {
                let var_value = node.crocoi(codegen)?.into_symbol(&self.code_pos)?;

                // type differs from annotation
                if let Some(var_type) = &self.var_type {
                    if !get_symbol_type(&var_value).eq(var_type) {
                        return Err(CrocoError::type_annotation_error(
                            &self.code_pos,
                            &self.left,
                        ));
                    }
                }

                var_value
            }

            // no node, use the defaut value
            None => match &self.var_type {
                None => return Err(CrocoError::infer_error(&self.code_pos, &self.left)),

                Some(var_type) => init_default(var_type, codegen, &self.code_pos)?,
            },
        };

        let variable = Rc::new(RefCell::new(value));
        codegen
            .symtable
            .insert_symbol(&self.left, variable)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(INodeResult::Void)
    }
}
