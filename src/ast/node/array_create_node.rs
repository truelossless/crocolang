use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;

#[cfg(feature = "crocoi")]
use {
    crate::crocoi::{symbol::get_symbol_type, symbol::Array, INodeResult, ISymTable, ISymbol},
    std::cell::RefCell,
    std::rc::Rc,
};

use crate::token::CodePos;

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
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        // don't allow empty array declarations
        // people should use
        // let a [num] and not let a [num] = []
        if self.contents.is_empty() {
            return Err(CrocoError::new(
                &self.code_pos,
                "do not use this syntax to declare empty arrays",
            )
            .hint("use type annotations to declare empty arrays"));
        }

        // visit all array elements
        let mut visited = Vec::with_capacity(self.contents.len());

        for el in &mut self.contents {
            visited.push(el.crocoi(symtable)?.into_symbol(&self.code_pos)?);
        }

        // infer the array type from the first element
        let array_type = get_symbol_type(&visited[0]);

        // make sure all elements are of the same type and wrap them in Rcs
        let mut visited_rc = Vec::with_capacity(self.contents.len());
        for el in visited.into_iter() {
            let el_type = get_symbol_type(&el);

            if !el_type.eq(&array_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "array elements must be of the same type",
                ));
            }

            visited_rc.push(Rc::new(RefCell::new(el)))
        }

        let array = Array {
            contents: visited_rc,
            array_type: Box::new(array_type),
        };

        Ok(NodeResult::Value(ISymbol::Array(array)))
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
