use crate::ast::{utils::init_default, AstNode, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{symbol_eq, SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

/// a node to declare a new variable (declared variable are initialized by default)
#[derive(Clone)]
pub struct VarDeclNode {
    // the var_name
    left: String,
    // the variable Assignement (None for a default assignment)
    right: Option<Box<dyn AstNode>>,
    // the type of the variable
    var_type: Symbol,
    code_pos: CodePos,
}

impl VarDeclNode {
    pub fn new(
        var_name: String,
        expr: Option<Box<dyn AstNode>>,
        var_type: Symbol,
        code_pos: CodePos,
    ) -> Self {
        VarDeclNode {
            left: var_name,
            right: expr,
            var_type,
            code_pos,
        }
    }
}

impl AstNode for VarDeclNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let value = match &mut self.right {
            // there is a node
            Some(node) => {
                let var_value = node.visit(symtable)?.into_symbol(&self.code_pos)?;

                // type differs from annotation
                if !self.var_type.is_void() && !symbol_eq(&var_value, &self.var_type) {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        format!(
                        "variable {} has been explicitely given a type but is declared with another one",
                        &self.left),
                    ));
                }

                // no annotation at all
                if var_value.is_void() && self.var_type.is_void() {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        format!("trying to assign a void expression to {}", self.left),
                    ));
                }

                var_value
            }

            // no node, use the defaut value
            None => {
                if self.var_type.is_void() {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        format!("cannot infer the type of the variable {}", self.left),
                    ));
                }

                init_default(&mut self.var_type, symtable, &self.code_pos)?;
                self.var_type.clone()
            }
        };

        symtable
            .insert_symbol(&self.left, value)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
    }
}
