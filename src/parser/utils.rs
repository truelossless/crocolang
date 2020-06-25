use super::Parser;

use crate::error::CrocoError;
use crate::token::{Token, CodePos, Token::*, SeparatorEnum::*, Identifier, LiteralEnum};

impl Parser {

    /// Expects a token
    pub fn expect_token(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        token: Token,
        error_msg: &str,
    ) -> Result<(), CrocoError> {
        // The EOF token is behaving like a newline in expect
        let mut next_token = self.next_token(iter);
        if next_token == Token::EOF {
            next_token = Separator(NewLine);
        }

        if next_token == token {
            Ok(())
        } else {
            Err(CrocoError::new(&self.token_pos, error_msg.to_owned()))
        }
    }

    /// Expects an identifier and returns its name
    pub fn expect_identifier(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        error_msg: &str,
    ) -> Result<Identifier, CrocoError> {
        match self.next_token(iter) {
            Identifier(identifier) => Ok(identifier),
            _ => Err(CrocoError::new(&self.token_pos, error_msg.to_owned())),
        }
    }

    /// Expects a literal string a returns its value
    pub fn expect_str(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        error_msg: &str,
    ) -> Result<String, CrocoError> {
        match self.next_token(iter) {
            Literal(LiteralEnum::Str(Some(s))) => Ok(s),
            _ => Err(CrocoError::new(&self.token_pos, error_msg.to_owned())),
        }
    }

    /// Discards all next tokens that are newlines
    pub fn discard_newlines(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) {
        while let Separator(NewLine) = self.peek_token(iter) {
            self.next_token(iter);
        }
    }
}