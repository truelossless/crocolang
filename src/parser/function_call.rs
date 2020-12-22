use super::{ExprParsingType::*, Parser};
use crate::ast::{node::FunctionCallNode, BackendNode};
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
    ) -> Result<Box<dyn BackendNode>, CrocoError> {
        let mut fn_args: Vec<Box<dyn BackendNode>> = Vec::new();

        let mut first_arg = false;

        loop {
            match self.peek_token(iter) {
                Separator(RightParenthesis) => {
                    self.next_token(iter);
                    break;
                }
                Separator(Comma) if first_arg => {
                    self.next_token(iter);
                }

                Separator(Comma) => {
                    return Err(CrocoError::new(&self.token_pos, "no argument before comma"))
                }

                _ if !first_arg => (),

                _ => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        &format!(
                            "expected a comma or a right parenthesis in {} function call",
                            identifier_name
                        ),
                    ))
                }
            }

            first_arg = true;

            self.discard_newlines(iter);
            fn_args.push(self.parse_expr(iter, AllowStructDeclaration)?)
        }

        Ok(Box::new(FunctionCallNode::new(
            identifier_name,
            fn_args,
            None,
            self.token_pos.clone(),
        )))
    }
}
