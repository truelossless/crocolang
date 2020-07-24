use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::CodePos;
/// a node holding a field of a struct
#[derive(Clone)]
pub struct DotFieldNode {
    field_name: String,
    bottom: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl DotFieldNode {
    pub fn new(field_name: String, code_pos: CodePos) -> Self {
        DotFieldNode {
            bottom: None,
            field_name,
            code_pos,
        }
    }
}

impl AstNode for DotFieldNode {

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!()
        }
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // LETS DO THIS FUNCTONAL STYLE !!! (just to try)
        // TODO: match so we can clone only the relevant field
        let field = self
            .bottom
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)?
            .borrow()
            .clone()
            .into_struct()
            .map_err(|_| {
                CrocoError::new(
                    &self.code_pos,
                    "expected a struct before the dot".to_owned(),
                )
            })?
            .fields
            .as_mut()
            .unwrap()
            .remove(&self.field_name)
            .ok_or_else(|| {
                CrocoError::new(&self.code_pos, format!("no field with the name {}", self.field_name))
            })?;

        Ok(NodeResult::Symbol(field))
    }
}
