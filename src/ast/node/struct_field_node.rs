use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::CodePos;
/// a node holding a field of a struct
#[derive(Clone)]
pub struct StructFieldNode {
    field_name: String,
    bottom: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl StructFieldNode {
    pub fn new(bottom: Box<dyn AstNode>, field_name: String, code_pos: CodePos) -> Self {
        StructFieldNode {
            bottom,
            field_name,
            code_pos,
        }
    }
}

impl AstNode for StructFieldNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // LETS DO THIS FUNCTONAL STYLE !!! (just to try)
        let field = self
            .bottom
            .visit(symtable)?
            .into_symbol(&self.code_pos)?
            .into_struct()
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "expected a struct before the dot".to_owned(),
                )
            })?
            .fields
            .unwrap()
            .remove(&self.field_name)
            .ok_or_else(|| {
                CrocoError::new(&self.code_pos, "no field with this name on {}".to_owned())
            })?;

        Ok(NodeResult::Symbol(field))
    }
}
