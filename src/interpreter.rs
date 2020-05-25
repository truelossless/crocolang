use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::symbol::SymTable;
use std::fs;

#[derive(Default)]
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

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), String> {
        let contents =
            fs::read_to_string(file_path).map_err(|_| format!("File not found: {}", file_path))?;

        self.exec(&contents)
    }

    pub fn exec(&mut self, code: &str) -> Result<(), String> {
        let tokens;
        let mut tree;

        match self.lexer.process(code) {
            Ok(t) => tokens = t,
            Err(e) => return Err(format!("Syntax error: {}", e)),
        }

        // println!("tokens: {:?}", &tokens);

        match self.parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(e) => return Err(format!("Parse error: {}", e)),
        }

        // println!("symbol tables: {:?}", self.symtable);
        // import the builtin library
        self.symtable.import_builtin_module("global");

        if let Err(e) = tree.visit(&mut self.symtable) {
            return Err(format!("Runtime error: {}", e));
        }

        // println!("symbol tables: {:?}", self.symtable);

        Ok(())
    }
}
