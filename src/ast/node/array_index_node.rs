use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent::*};
use crate::token::CodePos;

#[derive(Clone)]

/// a node used to access an array element at a certain index.
pub struct ArrayIndexNode {
    array: Option<Box<dyn AstNode>>,
    index: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl ArrayIndexNode {
    pub fn new(index: Box<dyn AstNode>, code_pos: CodePos) -> Self {
        ArrayIndexNode {
            array: None,
            index,
            code_pos,
        }
    }
}

impl AstNode for ArrayIndexNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.array.is_none() {
            self.array = Some(node);
        } else {
            unreachable!()
        }
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // visit the index node to get the number of the element to access
        let index_symbol = self.index.visit(symtable)?.into_symbol(&self.code_pos)?;

        let index = index_symbol
            .borrow()
            .clone()
            .into_primitive()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .into_num()
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        // get a mutable reference to the array, it should not fail on unwraps
        let array_ref = self
            .array
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)
            .unwrap();

        let array_borrow = array_ref
            .borrow_mut();

        let array = match &*array_borrow {
            Array(arr) => arr,

            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "expected an array".to_owned(),
                ))
            }
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
        match array.contents.as_ref().unwrap().get(index as usize) {
            Some(el) => Ok(NodeResult::Symbol(el.clone())),
            None => Err(CrocoError::new(
                &self.code_pos,
                "index out of bounds".to_owned(),
            )),
        }
    }
}
