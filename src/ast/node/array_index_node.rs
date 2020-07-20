use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol::*};
use crate::token::{CodePos};

#[derive(Clone)]

/// a node used to access an array element at a certain index.
pub struct ArrayIndexNode {
    name: String,
    index: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl ArrayIndexNode {
    pub fn new(name: String, index: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        ArrayIndexNode {
            name,
            index,
            code_pos,
        }
    }
}

impl AstNode for ArrayIndexNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {

        // visit the index node to get the number of the element to access
        let index = self
            .index
            .visit(symtable)?
            .into_symbol(&self.code_pos)?
            .into_primitive()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .into_num()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        // get a mutable reference to the array
        let array_symbol = symtable
            .get_mut_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        let array = match array_symbol {

            Array(arr) => arr,

            _ => return Err(CrocoError::new(
                &self.code_pos,
                "expected an array".to_owned())
            )

        };

        // make sure the index is a uint
        if index.fract() != 0.0 {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot use a floating number to index an array".to_owned(),
            ));
        }

        if index < 0.0 {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot use a negative index".to_owned(),
            ));
        }

        // return a clone to that element
        match array.contents.as_mut().unwrap().get(index as usize) {
            Some(el) => Ok(NodeResult::Symbol(el.clone())),
            None => Err(CrocoError::new(
                &self.code_pos,
                "index out of bounds".to_owned(),
            )),
        }
    }
}
