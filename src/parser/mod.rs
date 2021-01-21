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

use crate::error::CrocoError;
use crate::{
    ast::*,
    symbol::{FunctionDecl, StructDecl},
};
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, Token},
};
use std::{
    collections::{hash_map::Drain, HashMap},
    rc::Rc,
};

/// The croco parser
pub struct Parser {
    scope: BlockScope,
    /// the position of the parser relative to the source file
    token_pos: CodePos,
    /// The current token being parsed
    current_token: Token,
    /// The next token to be parsed
    next_token: Token,
    /// All the function declarations
    fn_decls: HashMap<String, FunctionDecl>,
    /// All the struct declarations
    struct_decls: HashMap<String, StructDecl>,
}

impl Parser {
    pub fn new() -> Self {
        let token_pos = CodePos {
            file: Rc::from(""),
            line: 0,
            word: 0,
        };

        Parser {
            scope: BlockScope::Keep,
            token_pos,
            current_token: Token::Discard,
            next_token: Token::Discard,
            fn_decls: HashMap::new(),
            struct_decls: HashMap::new(),
        }
    }

    /// Sets the scope of the parser
    pub fn set_scope(&mut self, scope: BlockScope) {
        self.scope = scope;
    }

    /// Builds an Abstact Syntax Tree (AST) with the results of the lexer.
    pub fn process(
        &mut self,
        tokens: Vec<(Token, CodePos)>,
    ) -> Result<Box<dyn BackendNode>, CrocoError> {
        // iterator which returns a movable and peekable token iterator
        let mut iter = tokens.into_iter().peekable();
        let root = self.parse_block(&mut iter, self.scope.clone(), true)?;
        Ok(root)
    }

    /// Checks if a struct or function with the same name has already been declared
    fn check_decls(&self, decl_name: &str) -> Result<(), CrocoError> {
        if self.fn_decls.contains_key(decl_name) {
            return Err(CrocoError::new(
                &self.token_pos,
                "function with the same name already declared",
            ));
        }

        if self.struct_decls.contains_key(decl_name) {
            return Err(CrocoError::new(
                &self.token_pos,
                "struct with the same name already declared",
            ));
        }

        Ok(())
    }

    pub fn register_fn_decl(
        &mut self,
        fn_name: &str,
        fn_decl: FunctionDecl,
    ) -> Result<(), CrocoError> {
        self.check_decls(fn_name)?;
        self.fn_decls.insert(fn_name.to_owned(), fn_decl);
        Ok(())
    }

    pub fn register_struct_decl(
        &mut self,
        struct_name: &str,
        struct_decl: StructDecl,
    ) -> Result<(), CrocoError> {
        self.check_decls(struct_name)?;
        self.struct_decls
            .insert(struct_name.to_owned(), struct_decl);
        Ok(())
    }

    /// Returns all the function declarations found by the parser
    pub fn take_fn_decls(&mut self) -> Drain<String, FunctionDecl> {
        self.fn_decls.drain()
    }

    /// Returns all the struct declarations found by the parser
    pub fn take_struct_decls(&mut self) -> Drain<String, StructDecl> {
        self.struct_decls.drain()
    }
}

/// An argument type and its name
#[derive(Clone, Debug)]
pub struct TypedArg {
    pub arg_name: String,
    pub arg_type: SymbolType,
}
/// defines if a struct declaration can be present in an expression
#[derive(PartialEq, Copy, Clone)]
pub enum ExprParsingType {
    AllowStructDeclaration,
    DenyStructDeclaration,
}
