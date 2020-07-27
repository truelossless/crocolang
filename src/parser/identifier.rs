use std::collections::HashMap;

use super::{ExprParsingType, Parser};

use crate::ast::node::*;
use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::parser::ExprParsingType::*;
use crate::token::{CodePos, Identifier, SeparatorEnum::*, Token, Token::*};

/// Parses an identifier in the right AstNode given the next tokens as the context
/// e.g tokens like identifier[0].name
impl Parser {
    // TODO: differenciate lvalues and rvalues
    pub fn parse_identifier(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        identifier: Identifier,
        parse_type: ExprParsingType,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        let mut chain_nodes: Vec<Box<dyn AstNode>> = Vec::new();

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

                    let field_name = self.expect_identifier(iter, "expected a field name")?;
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
                        "can't chain on struct creation".to_owned(),
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

        for mut node in chain_nodes.into_iter() {
            node.add_child(out_node);
            out_node = node;
        }

        Ok(out_node)
    }
}
