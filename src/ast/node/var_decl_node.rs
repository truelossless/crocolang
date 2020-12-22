use crate::ast::{AstNode, BackendNode};
use crate::{symbol_type::SymbolType, token::CodePos};

/// A node to declare a new variable (declared variable are initialized by default)
#[derive(Clone)]
pub struct VarDeclNode {
    // the var_name
    pub left: String,
    // the variable Assignement (None for a default assignment)
    pub right: Option<Box<dyn BackendNode>>,
    // the type of the variable
    pub var_type: Option<SymbolType>,
    pub code_pos: CodePos,
}

impl VarDeclNode {
    pub fn new(
        var_name: String,
        expr: Option<Box<dyn BackendNode>>,
        var_type: Option<SymbolType>,
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

impl AstNode for VarDeclNode {}
impl BackendNode for VarDeclNode {}
