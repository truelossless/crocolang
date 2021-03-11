use crate::{
    ast::node::ArrayCreateNode,
    crocol::{CrocolNode, LCodegen, LNodeResult, LSymbol},
    symbol_type::SymbolType,
    CrocoError,
};

impl CrocolNode for ArrayCreateNode {
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        if self.contents.is_empty() {
            return Err(CrocoError::empty_array_error(&self.code_pos));
        }

        let mut visited = Vec::with_capacity(self.contents.len());

        for el in &mut self.contents {
            visited.push(el.crocol(codegen)?.into_symbol(codegen, &self.code_pos)?);
        }

        let array_type = visited[0].symbol_type.clone();

        for el in visited.iter().skip(1) {
            if el.symbol_type != array_type {
                return Err(CrocoError::mixed_type_array(&self.code_pos));
            }
        }

        let alloca = codegen.alloc_array(visited);
        let load = codegen.builder.build_load(alloca, "loadarr");
        Ok(LNodeResult::Value(LSymbol {
            value: load,
            symbol_type: SymbolType::Array(Box::new(array_type)),
        }))
    }
}
