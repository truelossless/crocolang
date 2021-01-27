use symbol_type::SymbolType;

use crate::{
    ast::node::StructCreateNode,
    crocol::{
        utils::{get_or_define_struct, init_default},
        CrocolNode, LCodegen, LNodeResult, LSymbol,
    },
    symbol_type, CrocoError,
};

impl CrocolNode for StructCreateNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let struct_decl = codegen
            .symtable
            .get_struct_decl(&self.struct_type)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .clone();
        let struct_ty = get_or_define_struct(&self.struct_type, &struct_decl, codegen);

        let alloca = codegen.create_block_alloca(struct_ty.into(), "allocastruct");

        // initializes all fields
        for (i, (field_name, field_type)) in struct_decl.fields.iter().enumerate() {
            // get a pointer to the current field
            let field_ptr = codegen
                .builder
                .build_struct_gep(alloca, i as u32, "gepfield")
                .unwrap();

            // the field is present
            if let Some(node) = self.fields.get_mut(field_name) {
                let field_value = node.crocol(codegen)?.into_symbol(codegen, &self.code_pos)?;

                if field_value.symbol_type != *field_type {
                    return Err(CrocoError::field_type_error(field_name, &self.code_pos));
                }

                codegen.builder.build_store(field_ptr, field_value.value);

            // the field has not been declared
            } else {
                let field_symbol = LSymbol {
                    value: field_ptr.into(),
                    symbol_type: field_type.clone(),
                };

                init_default(&field_symbol, codegen);
            }
        }

        Ok(LNodeResult::Variable(LSymbol {
            value: alloca.into(),
            symbol_type: SymbolType::Struct(self.struct_type.clone()),
        }))
    }
}
