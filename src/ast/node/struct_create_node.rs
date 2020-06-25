use crate::ast::{AstNode, NodeResult, utils::init_default};
use crate::error::CrocoError;
use crate::symbol::{symbol_eq, Struct, SymTable, Symbol};
use crate::token::{CodePos};
use std::collections::HashMap;

/// a node holding a struct
#[derive(Clone)]
pub struct StructCreateNode {
    struct_type: String,
    fields: HashMap<String, Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl StructCreateNode {
    pub fn new(
        struct_type: String,
        fields: HashMap<String, Box<dyn AstNode>>,
        code_pos: CodePos,
    ) -> Self {
        StructCreateNode {
            struct_type,
            code_pos,
            fields,
        }
    }
}

// actually we can't move out as a node can be visited multiple times in a loop
impl AstNode for StructCreateNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        
        let mut struct_symbol = Struct {
            struct_type: self.struct_type.clone(),
            fields: Some(HashMap::new()),
        };

        // we need to check first if the struct is valid

        // get the struct declaration
        let struct_decl = symtable
            .get_struct_decl(&self.struct_type)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .clone();

        let struct_decl_len = struct_decl.len();

        // make sure all fields in struct decl are present
        for mut field_decl in struct_decl.into_iter() {
            let field_val = match self.fields.get_mut(&field_decl.0) {
                // this field has not been declared, use its default value
                None => {
                    init_default(&mut field_decl.1, symtable, &self.code_pos)?;
                    field_decl.1.clone()
                }

                // the field is present, visit it
                Some(field) => {
                    let field_val = field.visit(symtable)?.into_symbol(&self.code_pos)?;

                    if !symbol_eq(&field_decl.1, &field_val) {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            format!("field {} is not of the right type", field_decl.0),
                        ));
                    }

                    field_val
                }
            };

            struct_symbol
                .fields
                .as_mut()
                .unwrap()
                .insert(field_decl.0.clone(), field_val);
        }

        // also we have to make sure that there is no extra field in our struct
        if struct_decl_len != struct_symbol.fields.as_ref().unwrap().len() {
            return Err(CrocoError::new(
                &self.code_pos,
                "extra field in struct declaration".to_owned(),
            ));
        }

        Ok(NodeResult::Symbol(Symbol::Struct(struct_symbol)))
    }
}
