use crate::crocoi::CrocoiNode;
use crate::{ast::node::ArrayIndexNode, error::CrocoError};

#[cfg(feature = "crocoi")]
use crate::crocoi::{ICodegen, INodeResult, ISymbol::*};

impl CrocoiNode for ArrayIndexNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // visit the index node to get the number of the element to access
        let index_symbol = self.index.crocoi(codegen)?.into_symbol(&self.code_pos)?;

        let index = index_symbol
            .into_primitive()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .into_num()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        // get the variable referencing  the array, it should not fail on unwraps
        let array_ref = self
            .array
            .as_mut()
            .unwrap()
            .crocoi(codegen)?
            .into_var(&self.code_pos)?;

        let array_borrow = &mut *array_ref.borrow_mut();

        let array = match array_borrow {
            Array(arr) => arr,
            _ => return Err(CrocoError::wrong_type_indexing(&self.code_pos)),
        };

        if index < 0 {
            return Err(CrocoError::negative_indexing_error(&self.code_pos));
        }

        match array.contents.get(index as usize) {
            Some(el) => Ok(INodeResult::Variable(el.clone())),
            None => Err(CrocoError::index_out_of_bounds_error(&self.code_pos)),
        }
    }
}
