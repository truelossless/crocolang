use crate::ast::{AstNode};
use crate::error::CrocoError;
use crate::symbol::{get_symbol_type, SymTable};
use crate::{
    symbol_type::{type_eq, SymbolType},
    token::CodePos, crocoi::{INodeResult, ISymbol},
};
use crate::crocoi::symbol::{SymbolContent, Array};

/// a node representing an array symbol
/// checks at runtime if the type constraint is respected
#[derive(Clone)]
pub struct ArrayCreateNode {
    contents: Vec<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl ArrayCreateNode {
    pub fn new(contents: Vec<Box<dyn AstNode>>, code_pos: CodePos) -> Self {
        ArrayCreateNode { contents, code_pos }
    }
}

impl AstNode for ArrayCreateNode {
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        // visit all array elements
        let mut visited = Vec::new();

        for el in &mut self.contents {
            visited.push(el.crocoi(symtable)?.into_symbol(&self.code_pos)?);
        }

        // infer the array type from the first element
        let array_type = if visited.is_empty() {
            // we have no idea of the type since the array is empty
            SymbolType::Void
        } else {
            // the first element can be taken as the array type
            get_symbol_type(&*visited[0].borrow())
        };

        // make sure all elements are of the same type
        for el in visited.iter().skip(1) {
            let el_type = get_symbol_type(&*el.borrow());

            if !type_eq(&el_type, &array_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "array elements must be of the same type",
                ));
            }
        }

        let array = Array {
            contents: Some(visited),
            array_type: Box::new(array_type),
        };

        Ok(INodeResult::construct_symbol(SymbolContent::Array(array)))
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
