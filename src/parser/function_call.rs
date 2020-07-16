use super::{ExprParsingType::*, Parser};
use crate::ast::{node::FunctionCallNode, AstNode};
use crate::error::CrocoError;
use crate::token::SeparatorEnum::*;
use crate::token::{CodePos, Token, Token::*};

impl Parser {
    /// parses a function call
    /// warning: it does not consume the left parenthesis after the identifier name
    pub fn parse_function_call(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        identifier_name: String,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        let mut fn_args: Vec<Box<dyn AstNode>> = Vec::new();

        loop {
            // TODO: avoid hello()) function calls
            if let Separator(RightParenthesis) = self.peek_token(iter) {
                self.next_token(iter);
                break;
            }

            fn_args.push(self.parse_expr(iter, AllowStructDeclaration)?);
            self.discard_newlines(iter);

            match self.next_token(iter) {
                Separator(Comma) => (),
                Separator(RightParenthesis) => break,
                _ => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        format!("unexpected token in {} function call", identifier_name),
                    ));
                }
            }
        }

        Ok(Box::new(FunctionCallNode::new(
            identifier_name,
            fn_args,
            self.token_pos.clone(),
        )))
    }
}
