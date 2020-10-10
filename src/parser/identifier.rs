use std::collections::HashMap;

use super::{ExprParsingType, Parser};

use crate::ast::node::*;
use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::parser::ExprParsingType::*;
use crate::token::{CodePos, OperatorEnum::*, SeparatorEnum::*, Token, Token::*};

impl Parser {
    /// Parses an identifier in the right AstNode given the next tokens as the context
    /// e.g tokens like identifier[0].name
    pub fn parse_identifier(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        parse_type: ExprParsingType,
        // also returns if it's lvalue compatible, with a bool
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        // nodes to chain to
        let mut chain_nodes: Vec<Box<dyn AstNode>> = Vec::new();

        // refs / derefs happens after that
        let mut chain_ref_nodes: Vec<Box<dyn AstNode>> = Vec::new();

        // wether or not the expression is assignable

        // ref / deref as many times as needed
        loop {
            match self.peek_token(iter) {
                // ref
                Operator(BitwiseAnd) => {
                    chain_ref_nodes.push(Box::new(RefNode::new(self.token_pos.clone())));
                }

                // deref
                Operator(Multiplicate) => {
                    chain_ref_nodes.push(Box::new(DerefNode::new(self.token_pos.clone())))
                }

                _ => break,
            }

            self.next_token(iter);
        }

        // we can have either a literal or an identifier
        match self.next_token(iter) {
            Identifier(identifier) => {
                match self.peek_token(iter) {
                    // function call
                    Separator(LeftParenthesis) => {
                        self.next_token(iter);
                        chain_nodes.push(self.parse_function_call(iter, identifier.name)?);
                    }

                    // struct instanciation
                    Separator(LeftCurlyBracket) if parse_type == AllowStructDeclaration => {
                        self.next_token(iter);
                        self.discard_newlines(iter);

                        let mut fields: HashMap<String, Box<dyn AstNode>> = HashMap::new();

                        loop {
                            self.discard_newlines(iter);

                            if let Separator(RightCurlyBracket) = self.peek_token(iter) {
                                self.next_token(iter);
                                break;
                            }

                            let field_name =
                                self.expect_identifier(iter, "expected a field name")?;
                            self.expect_token(
                                iter,
                                Separator(Colon),
                                "expected a colon after the field name",
                            )?;
                            let field_expr = self.parse_expr(iter, AllowStructDeclaration)?;

                            fields.insert(field_name.name, field_expr);
                        }

                        if !chain_nodes.is_empty() {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                "can't chain on struct creation",
                            ));
                        }

                        return Ok(Box::new(StructCreateNode::new(
                            identifier.name,
                            fields,
                            self.token_pos.clone(),
                        )));
                    }

                    // anything else
                    _ => chain_nodes.push(Box::new(VarCallNode::new(
                        identifier.name,
                        self.token_pos.clone(),
                    ))),
                }
            }

            // primitive
            Literal(num) => {
                chain_nodes.push(Box::new(ConstantNode::new(num, self.token_pos.clone())))
            }

            // array literal
            Separator(LeftSquareBracket) => {
                chain_nodes.push(self.parse_array(iter)?)
            }

            _ => {
                return Err(CrocoError::new(
                    &self.token_pos,
                    "expected an identifier after the dereference operator",
                ))
            }
        }

        // from now on, we can chain fields with . and []
        loop {
            match self.peek_token(iter) {
                // struct instanciation

                // field or method call on a struct
                Separator(Dot) => {
                    self.next_token(iter);

                    let field = self
                        .expect_identifier(iter, "expected a field or method name after the dot")?;

                    // check if it's a method
                    if let Separator(LeftParenthesis) = self.peek_token(iter) {
                        self.next_token(iter);
                        chain_nodes.push(self.parse_function_call(iter, field.name.clone())?);
                    } else {
                        chain_nodes.push(Box::new(DotFieldNode::new(
                            field.name,
                            self.token_pos.clone(),
                        )));
                    }
                }

                // array indexing
                Separator(LeftSquareBracket) => {
                    self.next_token(iter);

                    let index = self.parse_expr(iter, DenyStructDeclaration)?;

                    self.expect_token(
                        iter,
                        Separator(RightSquareBracket),
                        "expected right square bracket after accessing an array",
                    )?;

                    chain_nodes.push(Box::new(ArrayIndexNode::new(index, self.token_pos.clone())));
                }

                // he next token doesn't belong here
                _ => break,
            }
        }
        // solve the chain
        let mut out_node = chain_nodes.remove(0);

        for mut node in chain_nodes
            .into_iter()
            .chain(chain_ref_nodes.into_iter().rev())
        {
            node.add_child(out_node);
            out_node = node;
        }

        Ok(out_node)
    }
}
