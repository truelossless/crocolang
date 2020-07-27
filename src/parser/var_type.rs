use super::Parser;

use crate::error::CrocoError;
use crate::symbol::{Array, Struct, SymbolContent};
use crate::token::{
    CodePos, KeywordEnum::*, LiteralEnum, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};

use std::cell::RefCell;
use std::rc::Rc;

/// Parses the type of a symbol
/// e.g [num], MyStruct, str
impl Parser {
    pub fn parse_var_type(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Result<SymbolContent, CrocoError> {
        match self.next_token(iter) {
            // primitive
            Keyword(Str) => Ok(SymbolContent::Primitive(LiteralEnum::Str(None))),
            Keyword(Num) => Ok(SymbolContent::Primitive(LiteralEnum::Num(None))),
            Keyword(Bool) => Ok(SymbolContent::Primitive(LiteralEnum::Bool(None))),

            // ref
            Operator(Ampersand) => Ok(SymbolContent::Ref(Rc::new(RefCell::new(
                self.parse_var_type(iter)?,
            )))),

            // struct
            Identifier(identifier) => Ok(SymbolContent::Struct(Struct {
                fields: None,
                struct_type: identifier.name,
            })),

            // array
            Separator(LeftSquareBracket) => {
                let ret = SymbolContent::Array(Array {
                    contents: None,
                    array_type: Box::new(self.parse_var_type(iter)?),
                });

                self.expect_token(
                    iter,
                    Separator(RightSquareBracket),
                    "expected a right bracket to close the array type",
                )?;

                Ok(ret)
            }

            _ => Err(CrocoError::new(
                &self.token_pos,
                "invalid variable type".to_owned(),
            )),
        }
    }
}
