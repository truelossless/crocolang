use std::collections::HashMap;

use super::{ExprParsingType, Parser};

use crate::ast::node::*;
use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::parser::ExprParsingType::*;
use crate::token::{CodePos, Identifier, SeparatorEnum::*, Token, Token::*};

/// Parses an identifier into either a FunctionCallNode or a VariableNode
impl Parser {
    pub fn parse_identifier(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        identifier: Identifier,
        parse_type: ExprParsingType,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        let peek_token = self.peek_token(iter);

        match peek_token {
            // function call
            Separator(LeftParenthesis) => {
                self.next_token(iter);
                Ok(self.parse_function_call(iter, identifier.name)?)
            }
            // struct instanciation
            Separator(LeftBracket) if parse_type == AllowStructDeclaration => {
                self.next_token(iter);
                self.discard_newlines(iter);

                let mut fields: HashMap<String, Box<dyn AstNode>> = HashMap::new();

                loop {
                    self.discard_newlines(iter);

                    if let Separator(RightBracket) = self.peek_token(iter) {
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

                Ok(Box::new(StructCreateNode::new(
                    identifier.name,
                    fields,
                    self.token_pos.clone(),
                )))
            }

            // field call on a struct
            Separator(Dot) => {
                let mut out_node: Box<dyn AstNode> = Box::new(VarCallNode::new(
                    identifier.get_namespaced_name(),
                    self.token_pos.clone(),
                ));

                while let Separator(Dot) = self.peek_token(iter) {
                    self.next_token(iter);

                    let field =
                        self.expect_identifier(iter, "expected a field name after the dot")?;
                    out_node = Box::new(StructFieldNode::new(
                        out_node,
                        field.name,
                        self.token_pos.clone(),
                    ));
                }

                Ok(out_node)
            }

            // variable call
            _ => Ok(Box::new(VarCallNode::new(
                identifier.name,
                self.token_pos.clone(),
            ))),
        }
    }
}
