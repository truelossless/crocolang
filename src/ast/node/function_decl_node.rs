use crate::ast::{AstNode, BlockScope, NodeResult};
use crate::ast::node::BlockNode;

use crate::parser::TypedArg;
use crate::symbol::{Decl, FunctionDecl, FunctionKind, SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

use crate::error::CrocoError;

/// function declaration node
#[derive(Clone)]
pub struct FunctionDeclNode {
    name: String,
    return_type: Option<Symbol>,
    args: Option<Vec<TypedArg>>,
    body: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl FunctionDeclNode {
    pub fn new(
        name: String,
        return_type: Symbol,
        args: Vec<TypedArg>,
        code_pos: CodePos,
    ) -> Self {
        FunctionDeclNode {
            name,
            return_type: Some(return_type),
            args: Some(args),
            body: Some(Box::new(BlockNode::new(BlockScope::New))),
            code_pos,
        }
    }
}

impl AstNode for FunctionDeclNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {

        // once the function is declared we can move out its content since this node is not going to be used again
        let body = std::mem::replace(&mut self.body, None).unwrap();
        let args = std::mem::replace(&mut self.args, None).unwrap();
        let name = std::mem::replace(&mut self.name, String::new());
        let return_type = std::mem::replace(&mut self.return_type, None).unwrap();

        let fn_decl = FunctionDecl::new(args, return_type, FunctionKind::Regular(body));

        symtable.register_decl(name, Decl::FunctionDecl(fn_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;
        Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        self.body = Some(node);
    }
}