mod array;
mod block;
mod expr;
mod function_call;
mod function_decl;
mod identifier;
mod iter;
mod node;
mod utils;
mod var_type;

use crate::ast::*;
use crate::error::CrocoError;
use crate::symbol::SymbolContent;
use crate::token::{CodePos, Token};
use std::rc::Rc;

pub struct Parser {
    scope: BlockScope,
    token_pos: CodePos,
    next_token_pos: CodePos,
    current_token: Token,
    next_token: Token,
}

impl Parser {
    pub fn new() -> Self {
        let token_pos = CodePos {
            file: Rc::from(""),
            line: 0,
            word: 0,
        };

        Parser {
            scope: BlockScope::New,
            next_token_pos: token_pos.clone(),
            token_pos,
            current_token: Token::Discard,
            next_token: Token::Discard,
        }
    }

    pub fn set_scope(&mut self, scope: BlockScope) {
        self.scope = scope;
    }

    /// builds an Abstact Syntax Tree (AST) with the results of the lexer.
    pub fn process(
        &mut self,
        tokens: Vec<(Token, CodePos)>,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        // iterator which returns a movable and peekable token iterator
        let mut iter = tokens.into_iter().peekable();
        let root = self.parse_block(&mut iter, self.scope.clone(), true)?;
        Ok(root)
    }
}

#[derive(Clone, Debug)]
pub struct TypedArg {
    pub arg_name: String,
    pub arg_type: SymbolContent,
}

/// defines if a struct declaration can be present in an expression
#[derive(PartialEq, Copy, Clone)]
pub enum ExprParsingType {
    AllowStructDeclaration,
    DenyStructDeclaration,
}
