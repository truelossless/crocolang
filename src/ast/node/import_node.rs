use std::fs;
use unicode_segmentation::UnicodeSegmentation;

use crate::ast::{AstNode, BlockScope, NodeResult};
use crate::error::CrocoError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::symbol::{SymTable, Symbol};
use crate::token::{CodePos, LiteralEnum::*};

/// a node to import code from another module, at runtime.
#[derive(Clone)]
pub struct ImportNode {
    name: String,
    bottom: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl ImportNode {
    pub fn new(name: String, code_pos: CodePos) -> Self {
        ImportNode {
            name,
            bottom: None,
            code_pos,
        }
    }
}

impl AstNode for ImportNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // we have a relative path e.g import "./my_module"
        // look for a file with this name
        if self.name.starts_with('.') {
            let file_contents =
                fs::read_to_string(format!("{}.croco", self.name)).map_err(|_| {
                    CrocoError::new(
                        &self.code_pos,
                        format!("cannot find the file {}.croco", self.name),
                    )
                })?;

            // lex the new import
            // namespace everything created there with the import name
            let mut lexer = Lexer::new();

            // find the real import name
            // e.g "./module/me/love" => "love"
            let import_name = self.name.split_word_bounds().last().unwrap();

            // import name should be the real import name now.
            lexer.set_namespace(import_name.to_owned());
            let tokens = lexer.process(&file_contents)?;

            // .. and resolve to an AST the import
            // TODO: export only when pub is used
            let mut parser = Parser::new();

            // we can now add the import as a closure:
            // a block node which doesn't introduce a new scope
            parser.set_scope(BlockScope::Keep);
            let mut bottom = parser.process(tokens)?;
            bottom.visit(symtable)?;
            self.bottom = Some(bottom);

            Ok(NodeResult::Symbol(Symbol::Primitive(Void)))

        // we have an absolute path e.g import "math"
        // we are looking for a builtin module with this name
        } else {
            // check if the module part of the std library
            if symtable.import_builtin_module(&self.name) {
                Ok(NodeResult::Symbol(Symbol::Primitive(Void)))
            } else {
                Err(CrocoError::new(
                    &self.code_pos,
                    format!("{} module not found in the builtin library", self.name),
                ))
            }
        }
    }
}
