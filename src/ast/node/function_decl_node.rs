use crate::{ast::AstNode, symbol::FunctionDecl};
use crate::{ast::BackendNode, token::CodePos};

/// Function declaration node
#[derive(Clone)]
pub struct FunctionDeclNode {
    pub name: String,
    pub fn_decl: Option<FunctionDecl>,
    pub fn_body: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl FunctionDeclNode {
    pub fn new(
        name: String,
        fn_decl: FunctionDecl,
        fn_body: Box<dyn BackendNode>,
        code_pos: CodePos,
    ) -> Self {
        FunctionDeclNode {
            name,
            fn_decl: Some(fn_decl),
            fn_body: Some(fn_body),
            code_pos,
        }
    }
}

impl AstNode for FunctionDeclNode {}
impl BackendNode for FunctionDeclNode {}
