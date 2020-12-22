use crate::{ast::AstNode, symbol_type::SymbolType};
use crate::{ast::BackendNode, token::CodePos};

// A node holding a type value
#[derive(Clone)]
pub struct TypeNode {
    pub value: SymbolType,
    pub code_pos: CodePos,
}

impl TypeNode {
    pub fn new(value: SymbolType, code_pos: CodePos) -> Self {
        TypeNode { value, code_pos }
    }
}

impl AstNode for TypeNode {}
impl BackendNode for TypeNode {}
