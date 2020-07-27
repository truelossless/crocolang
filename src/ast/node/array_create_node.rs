use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{get_symbol_type, symbol_eq, Array, SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum};

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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // visit all array elements
        let mut visited = Vec::new();

        for el in &mut self.contents {
            visited.push(el.visit(symtable)?.into_symbol(&self.code_pos)?);
        }

        // infer the array type from the first element
        let array_type = if visited.is_empty() {
            // we have no idea of the type since the array is empty
            SymbolContent::Primitive(LiteralEnum::Void)
        } else {
            // the first element can be taken as the array type
            get_symbol_type(visited[0].clone())
        };

        // make sure all elements are of the same type
        for el in visited.iter().skip(1) {
            if !symbol_eq(&*el.borrow(), &array_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "array elements must be of the same type".to_owned(),
                ));
            }
        }

        let array = Array {
            contents: Some(visited),
            array_type: Box::new(array_type),
        };

        Ok(NodeResult::construct_symbol(SymbolContent::Array(array)))
    }
}
