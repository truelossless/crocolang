use crate::{
    ast::node::DotFieldNode,
    crocol::{utils::auto_deref, CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for DotFieldNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let mut struct_ptr = self
            .bottom
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_pointer(codegen, &self.code_pos)?;

        struct_ptr = auto_deref(struct_ptr, codegen);

        match struct_ptr.symbol_type {
            SymbolType::Struct(struct_name) => {
                let struct_decl = codegen
                    .symtable
                    .get_struct_decl(&struct_name)
                    .map_err(|e| CrocoError::new(&self.code_pos, e))?;

                let index = struct_decl
                    .fields
                    .keys()
                    .position(|field_name| field_name == &self.field_name)
                    .ok_or_else(|| CrocoError::no_field_error(&self.field_name, &self.code_pos))?;

                let field_ptr = codegen
                    .builder
                    .build_struct_gep(
                        struct_ptr.value.into_pointer_value(),
                        index as u32,
                        "gepdotfield",
                    )
                    .unwrap();

                Ok(LNodeResult::Variable(LSymbol {
                    value: field_ptr.into(),
                    symbol_type: struct_decl.fields.get(&self.field_name).unwrap().clone(),
                }))
            }

            _ => unimplemented!(),
        }
    }
}
