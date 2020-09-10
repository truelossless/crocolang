use super::{Parser, TypedArg};
use crate::ast::BlockScope;
use crate::error::CrocoError;
use crate::symbol::{FunctionDecl, FunctionKind};
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, SeparatorEnum::*, Token, Token::*},
};

impl Parser {
    /// parses a function declation into a FunctionDecl
    /// warning: does not consume the fn keyword
    pub fn parse_function_decl(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Result<(FunctionDecl, String), CrocoError> {
        let identifier = self.expect_identifier(
            iter,
            "expected the function name after function declaration",
        )?;

        self.expect_token(
            iter,
            Separator(LeftParenthesis),
            "expected a left parenthensis after the function name",
        )?;

        let mut typed_args: Vec<TypedArg> = Vec::new();

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
                        format!(
                            "expected a comma or a right parenthesis in {} function declaration",
                            identifier.name
                        ),
                    ))
                }
            }

            first_arg = true;

            self.discard_newlines(iter);

            // we're expecting an argument variable name here
            let arg_name =
                self.expect_identifier(iter, "expected an argument name in function declaration")?;

            // here this should be the argument type
            let arg_type = self.parse_var_type(iter)?;
            typed_args.push(TypedArg {
                arg_name: arg_name.name,
                arg_type,
            });
        }

        // Might allow weird parsing: does it matter ?
        // fn bla()
        // Void
        // { ...
        self.discard_newlines(iter);

        // if the return type isn't specified the function is Void
        let return_type = if let Separator(LeftCurlyBracket) = self.peek_token(iter) {
            SymbolType::Void
        } else {
            self.parse_var_type(iter)?
        };

        self.discard_newlines(iter);

        self.expect_token(
            iter,
            Separator(LeftCurlyBracket),
            "expected a left bracket after function declaration",
        )?;

        // get the namespaced name of the function
        // let fn_name = identifier.get_namespaced_name();
        let fn_decl = FunctionDecl {
            args: typed_args,
            return_type,
            body: Some(FunctionKind::Regular(self.parse_block(
                iter,
                BlockScope::New,
                false,
            )?)),
        };

        Ok((fn_decl, identifier.name))
    }
}
