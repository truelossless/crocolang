use crate::error::{CrocoError, CrocoErrorKind};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::symbol::SymTable;
use crate::token::CodePos;
use std::{fs, rc::Rc};

pub struct Interpreter {
    lexer: Lexer,
    parser: Parser,
    symtable: SymTable,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            parser: Parser::new(),
            lexer: Lexer::new(),
            // create the symbol tables and add the global scope
            symtable: SymTable::new(),
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
                format!("file not found: {}", file_path),
            );
            err.set_kind(CrocoErrorKind::IO);
            err
        })?;

        self.lexer.set_file(file_path);
        self.exec(&contents)
    }

    pub fn exec(&mut self, code: &str) -> Result<(), CrocoError> {
        // empty the symtable
        self.symtable = SymTable::new();

        let tokens;
        let mut tree;

        match self.lexer.process(code) {
            Ok(t) => tokens = t,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Syntax);
                return Err(e);
            }
        }

        // println!("tokens: {:?}", &tokens);

        match self.parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Parse);
                return Err(e);
            }
        }

        // import the builtin library
        self.symtable.import_builtin_module("global");
        self.symtable.import_builtin_module("os");

        // println!("symbol tables: {:?}", self.symtable);

        if let Err(mut e) = tree.visit(&mut self.symtable) {
            e.set_kind(CrocoErrorKind::Runtime);
            return Err(e);
        }

        // println!("symbol tables: {:?}", self.symtable);

        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new()
    }
}
