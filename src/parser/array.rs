use super::Parser;

use crate::ast::node::*;
use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::parser::ExprParsingType::*;
use crate::token::{CodePos, SeparatorEnum::*, Token, Token::*};

impl Parser {
    /// Parses an array literal to the corresponding symbol node.
    /// e.g [3+3, 5, 8]
    /// warning: does not consume the opening square bracket
    pub fn parse_array(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        let mut symbols: Vec<Box<dyn AstNode>> = Vec::new();

        loop {
            if let Separator(RightSquareBracket) = self.peek_token(iter) {
                self.next_token(iter);
                break;
            }

            symbols.push(self.parse_expr(iter, AllowStructDeclaration)?);
            self.discard_newlines(iter);

            match self.next_token(iter) {
                Separator(Comma) => (),
                Separator(RightSquareBracket) => break,
                _ => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        "unexpected token in {} array declaration".to_owned(),
                    ));
                }
            }
        }

        // we can't construct directly a SymbolNode for arrays, since their types must be dynamically checked at runtime.
        // e.g disallow ["hi !", a] where a is a number
        // the parser can't know it's invalid
        Ok(Box::new(ArrayCreateNode::new(
            symbols,
            self.token_pos.clone(),
        )))
    }
}
