use crate::crocoi::CrocoiNode;
use crate::{ast::node::StructCreateNode, error::CrocoError};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::crocoi::{
    symbol::get_symbol_type, symbol::Struct, utils::init_default, ICodegen, INodeResult, ISymbol,
};

impl CrocoiNode for StructCreateNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let mut struct_symbol = Struct {
            struct_type: self.struct_type.clone(),
            fields: HashMap::new(),
        };

        // we need to check first if the struct is valid

        // get the struct declaration
        let struct_decl = codegen
            .symtable
            .get_struct_decl(&self.struct_type)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .clone();

        let struct_decl_len = struct_decl.fields.len();

        // make sure all fields in struct decl are present
        for field_decl in struct_decl.fields.into_iter() {
            let field_val = match self.fields.get_mut(&field_decl.0) {
                // this field has not been declared, use its default value
                None => Rc::new(RefCell::new(init_default(
                    &field_decl.1,
                    codegen,
                    &self.code_pos,
                )?)),

                // the field is present, visit it
                Some(field) => {
                    let field_val = field.crocoi(codegen)?.into_value(&self.code_pos)?;

                    if field_decl.1 != get_symbol_type(&field_val) {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            &format!("field {} is not of the right type", field_decl.0),
                        ));
                    }

                    Rc::new(RefCell::new(field_val))
                }
            };

            struct_symbol.fields.insert(field_decl.0.clone(), field_val);
        }

        // also we have to make sure that there is no extra field in our struct
        if struct_decl_len != struct_symbol.fields.len() {
            return Err(CrocoError::new(
                &self.code_pos,
                "extra field in struct declaration",
            ));
        }

        Ok(INodeResult::Value(ISymbol::Struct(struct_symbol)))
    }
}
