use super::Parser;

use crate::error::CrocoError;
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, KeywordEnum::*, OperatorEnum::*, SeparatorEnum::*, Token, Token::*},
};

/// Parses the type of a symbol
/// e.g [num], MyStruct, str
impl Parser {
    pub fn parse_var_type(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Result<SymbolType, CrocoError> {
        match self.next_token(iter) {
            // primitive
            Keyword(Str) => Ok(SymbolType::Str),
            Keyword(Fnum) => Ok(SymbolType::Fnum),
            Keyword(Num) => Ok(SymbolType::Num),
            Keyword(Bool) => Ok(SymbolType::Bool),

            // ref
            Operator(BitwiseAnd) => Ok(SymbolType::Ref(Box::new(self.parse_var_type(iter)?))),

            // struct
            Identifier(identifier) => Ok(SymbolType::Struct(identifier.name)),

            // array
            Separator(LeftSquareBracket) => {
                let ret = SymbolType::Array(Box::new(self.parse_var_type(iter)?));

                self.expect_token(
                    iter,
                    Separator(RightSquareBracket),
                    "expected a right bracket to close the array type",
                )?;

                Ok(ret)
            }

            _ => Err(CrocoError::new(&self.token_pos, "invalid variable type")),
        }
    }
}
