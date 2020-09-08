use super::Parser;

use crate::ast::node::*;
use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::{
    symbol_type::SymbolType,
    token::{KeywordEnum::*, OperatorEnum::*, Token, Token::*},
};

impl Parser {
    /// util to build a node from a token
    pub fn get_node(&self, token: Token) -> Result<Box<dyn AstNode>, CrocoError> {
        // println!("got token {:?}", token);

        let code_pos = self.token_pos.clone();

        match token {
            Identifier(identifier) => Ok(Box::new(VarCopyNode::new(
                identifier.name,
                self.token_pos.clone(),
            ))),
            // since the parser has no idea of the backend, we cannot construct a SymbolNode directly.
            Literal(literal) => Ok(Box::new(ConstantNode::new(literal, code_pos))),
            Keyword(Num) => Ok(Box::new(TypeNode::new(SymbolType::Num, code_pos))),
            Keyword(Str) => Ok(Box::new(TypeNode::new(SymbolType::Str, code_pos))),
            Keyword(Bool) => Ok(Box::new(TypeNode::new(SymbolType::Bool, code_pos))),
            Operator(Plus) => Ok(Box::new(PlusNode::new(code_pos))),
            Operator(Minus) => Ok(Box::new(MinusNode::new(code_pos))),
            Operator(UnaryMinus) => Ok(Box::new(UnaryMinusNode::new(code_pos))),
            Operator(Multiplicate) => Ok(Box::new(MultiplicateNode::new(code_pos))),
            Operator(Divide) => Ok(Box::new(DivideNode::new(code_pos))),
            Operator(Power) => Ok(Box::new(PowerNode::new(code_pos))),
            Operator(Equals) => Ok(Box::new(CompareNode::new(Equals, code_pos))),
            Operator(NotEquals) => Ok(Box::new(CompareNode::new(NotEquals, code_pos))),
            Operator(GreaterOrEqual) => Ok(Box::new(CompareNode::new(GreaterOrEqual, code_pos))),
            Operator(GreaterThan) => Ok(Box::new(CompareNode::new(GreaterThan, code_pos))),
            Operator(LowerOrEqual) => Ok(Box::new(CompareNode::new(LowerOrEqual, code_pos))),
            Operator(LowerThan) => Ok(Box::new(CompareNode::new(LowerThan, code_pos))),
            Operator(Bang) => Ok(Box::new(NotNode::new(code_pos))),
            Operator(As) => Ok(Box::new(AsNode::new(code_pos))),
            _ => Err(CrocoError::new(
                &self.token_pos,
                &format!("can't evaluate token in expression: {:?}", token),
            )),
        }
    }

    /// util to add a node to the output
    pub fn add_node(
        &mut self,
        output: &mut Vec<Box<dyn AstNode>>,
        token: Token,
    ) -> Result<(), CrocoError> {
        let pos = self.token_pos.clone();
        let mut root_node = self.get_node(token)?;

        let right = match output.pop() {
            Some(x) => x,
            None => return Err(CrocoError::new(&pos, "missing element in expression")),
        };

        // if we have a binary node we must get two elements on the output
        if let AstNodeType::BinaryNode = root_node.get_type() {
            let left = match output.pop() {
                Some(x) => x,
                None => return Err(CrocoError::new(&pos, "missing element in expression")),
            };

            root_node.add_child(left);
        }

        root_node.add_child(right);

        output.push(root_node);
        Ok(())
    }
}
