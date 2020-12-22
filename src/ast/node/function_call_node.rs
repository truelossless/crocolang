use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;

#[derive(Clone)]
pub struct FunctionCallNode {
    pub fn_name: String,
    pub args: Vec<Box<dyn BackendNode>>,
    // wether or not this is a method (we have to inject "self" in this case)
    pub method: Option<Box<dyn BackendNode>>,
    pub code_pos: CodePos,
}

impl FunctionCallNode {
    pub fn new(
        fn_name: String,
        args: Vec<Box<dyn BackendNode>>,
        method: Option<Box<dyn BackendNode>>,
        code_pos: CodePos,
    ) -> Self {
        FunctionCallNode {
            fn_name,
            args,
            method,
            code_pos,
        }
    }
}

impl AstNode for FunctionCallNode {
    fn add_child(&mut self, node: Box<dyn BackendNode>) {
        if self.method.is_none() {
            self.method = Some(node);
        } else {
            unreachable!()
        }
    }
}

impl BackendNode for FunctionCallNode {}
