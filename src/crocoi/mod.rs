pub mod symbol;
use symbol::import_builtin_module;

pub use self::symbol::INodeResult;
pub use self::symbol::ISymbol;

pub mod node;
pub mod utils;

use crate::error::{CrocoError, CrocoErrorKind};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::symbol::SymTable;
use crate::token::CodePos;
use std::{fs, rc::Rc};

pub struct Crocoi {
    file_path: String,
}

impl Crocoi {
    pub fn new() -> Self {
        Crocoi {
            file_path: String::new(),
        }
    }

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), CrocoError> {
        let contents = fs::read_to_string(file_path).map_err(|_| {
            let mut err = CrocoError::new(
                &CodePos {
                    file: Rc::from(file_path),
                    line: 0,
                    word: 0,
                },
                &format!("file not found: {}", file_path),
            );
            err.set_kind(CrocoErrorKind::IO);
            err
        })?;

        self.file_path = file_path.to_owned();
        self.exec(&contents)
    }

    pub fn exec(&mut self, code: &str) -> Result<(), CrocoError> {
        let tokens;
        let mut tree;

        let mut lexer = Lexer::new();
        lexer.set_file(&self.file_path);
        match lexer.process(code) {
            Ok(t) => tokens = t,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Syntax);
                return Err(e);
            }
        }

        // println!("tokens: {:?}", &tokens);
        let mut parser = Parser::new();
        match parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Parse);
                return Err(e);
            }
        }

        // import the builtin library
        let mut symtable = SymTable::new();
        import_builtin_module(&mut symtable, "global");

        // println!("symbol tables: {:?}", self.symtable);
        if let Err(mut e) = tree.crocoi(&mut symtable) {
            e.set_kind(CrocoErrorKind::Runtime);
            return Err(e);
        }

        // println!("symbol tables: {:?}", self.symtable);

        Ok(())
    }
}

impl Default for Crocoi {
    fn default() -> Self {
        Crocoi::new()
    }
}
