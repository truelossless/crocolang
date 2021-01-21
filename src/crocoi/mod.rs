pub mod node;
pub mod stdlib;
pub mod symbol;
pub mod utils;
use dyn_clonable::clonable;
use symbol::import_builtin_module;

pub use self::symbol::ICodegen;
pub use self::symbol::INodeResult;
pub use self::symbol::ISymbol;

use crate::ast::{node::FunctionCallNode, AstNode};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::symbol::SymTable;
use crate::token::CodePos;
use crate::{
    error::{CrocoError, CrocoErrorKind},
    symbol::Decl,
};
use std::{collections::HashMap, fs};

#[clonable]
pub trait CrocoiNode: AstNode + Clone {
    /// crocoi backend interpreter
    fn crocoi(&mut self, _codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        unimplemented!();
    }
}

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
            CrocoError::from_type(format!("file not found: {}", file_path), CrocoErrorKind::Io)
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
                e.set_kind_if_unknown(CrocoErrorKind::Syntax);
                return Err(e);
            }
        }

        // println!("tokens: {:?}", &tokens);
        let mut parser = Parser::new();
        match parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(mut e) => {
                e.set_kind_if_unknown(CrocoErrorKind::Parse);
                return Err(e);
            }
        }

        // the first element in the tree should be a BlockNode.
        // add a call for the main function in it
        let code_pos = CodePos {
            file: lexer.get_file(),
            line: 0,
            word: 0,
        };

        tree.add_child(Box::new(FunctionCallNode::new(
            "main".to_owned(),
            Vec::new(),
            None,
            code_pos,
        )));

        let mut codegen = ICodegen {
            functions: HashMap::new(),
            symtable: SymTable::new(),
        };

        // import the builtin library
        import_builtin_module(&mut codegen, "global");

        // import all the declarations found by the parser
        for (fn_name, fn_decl) in parser.take_fn_decls() {
            codegen
                .symtable
                .register_decl(fn_name, Decl::FunctionDecl(fn_decl))
                .map_err(|e| CrocoError::from_type(e, CrocoErrorKind::Runtime))?;
        }

        for (struct_name, struct_decl) in parser.take_struct_decls() {
            codegen
                .symtable
                .register_decl(struct_name, Decl::StructDecl(struct_decl))
                .unwrap();
        }

        // println!("symbol tables: {:?}", self.symtable);
        if let Err(mut e) = tree.crocoi(&mut codegen) {
            e.set_kind_if_unknown(CrocoErrorKind::Runtime);
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
