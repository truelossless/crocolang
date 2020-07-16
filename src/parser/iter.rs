use super::Parser;

use crate::token::{CodePos, Token};

impl Parser {
    /// get the next token
    pub fn next_token(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Token {
        // we haven't peeked the field yet
        if let Token::Discard = self.next_token {
            match iter.next() {
                Some((next, code_pos)) => {
                    self.current_token = next;
                    self.token_pos = code_pos;
                }
                None => {
                    self.current_token = Token::EOF;
                }
            }

        // we have peeked the field so we can reuse the old values
        } else {
            self.current_token = std::mem::replace(&mut self.next_token, Token::Discard);
            self.token_pos = self.next_token_pos.clone();
        }

        self.next_token = Token::Discard;
        self.current_token.clone()
    }

    /// get the next token without advancing the iterator
    pub fn peek_token(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Token {
        // if we haven't peeked yet get the token
        if let Token::Discard = self.next_token {
            match iter.next() {
                Some((peek, code_pos)) => {
                    self.next_token = peek;
                    self.next_token_pos = code_pos;
                }
                None => {
                    self.next_token = Token::EOF;
                }
            }
        }

        self.next_token.clone()
    }
}
