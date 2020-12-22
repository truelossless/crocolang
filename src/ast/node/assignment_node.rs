use crate::ast::{AstNode, BackendNode};
use crate::token::CodePos;
/// A node to assign a variable to a certain value
#[derive(Clone)]
pub struct AssignmentNode {
    // variable to assign to (a VarRefNode)
    pub var: Box<dyn BackendNode>,
    // expr assigned
    pub expr: Box<dyn BackendNode>,
    pub code_pos: CodePos,
}

impl AssignmentNode {
    pub fn new(var: Box<dyn BackendNode>, expr: Box<dyn BackendNode>, code_pos: CodePos) -> Self {
        AssignmentNode {
            var,
            expr,
            code_pos,
        }
    }
}

impl AstNode for AssignmentNode {}
impl BackendNode for AssignmentNode {}
