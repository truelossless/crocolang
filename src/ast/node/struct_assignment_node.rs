use crate::ast::{AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{symbol_eq, SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

/// a node to assign a struct field to a certain value
// this is a little bit hacky, and I hope to resolve that someday and merge this with AssignmentNode
#[derive(Clone)]
pub struct StructAssignmentNode {
    // struct to assign to
    name: String,
    // the succeding fields to access the desired field
    fields: Vec<String>,
    // expr assigned
    expr: Box<dyn AstNode>,
    code_pos: CodePos,
}

impl StructAssignmentNode {
    pub fn new(
        name: String,
        fields: Vec<String>,
        expr: Box<dyn AstNode>,
        code_pos: CodePos,
    ) -> Self {
        StructAssignmentNode {
            name,
            fields,
            expr,
            code_pos,
        }
    }
}

impl AstNode for StructAssignmentNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let expr_value = self.expr.visit(symtable)?.into_symbol(&self.code_pos)?;

        let mut current_struct_symbol = symtable
            .get_mut_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        for field in &self.fields {
            match current_struct_symbol {
                Symbol::Struct(s) => {
                    current_struct_symbol =
                        s.fields.as_mut().unwrap().get_mut(field).ok_or_else(|| {
                            CrocoError::new(
                                &self.code_pos,
                                format!("this field doesn't exist on {}", field),
                            )
                        })?;
                }

                _ => {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        format!(
                            "cannot access the field {} because it's not a struct",
                            self.name
                        ),
                    ))
                }
            }
        }

        // now current_struct_symbol contains the field we want to modify
        if !symbol_eq(current_struct_symbol, &expr_value) {
            return Err(CrocoError::new(
                &self.code_pos,
                "cannot change the type of a struct field".to_owned(),
            ));
        }

        *current_struct_symbol = expr_value;

        Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
    }
}
