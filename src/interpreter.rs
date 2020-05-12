use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::RwLock;

use crate::lexer::Lexer;
use crate::token::LiteralEnum;

use crate::parser::Parser;
use crate::parser::TypedArg;

use crate::ast::BuiltinCallback;
use crate::ast::Symbol;
use crate::ast::FunctionCall;
use crate::ast::FunctionKind;

pub type SymbolTable = Arc<RwLock<Vec<HashMap<String, Symbol>>>>;

fn println(vars: Vec<LiteralEnum>) -> Result<Option<LiteralEnum>, String> {
    
    let arg = match &vars[0] {
        LiteralEnum::Text(Some(t)) => t,
        _ => unreachable!()
    };

    println!("{}", arg);
    Ok(None)
}

#[derive(Default)]
pub struct Interpreter {
    lexer: Lexer,
    parser: Parser,
    // we use an array of symbol tables here to handle nested tables.
    symbol_tables: SymbolTable,
}

impl<'a> Interpreter {
    pub fn new() -> Self {
        Interpreter {
            parser: Parser::new(),
            lexer: Lexer::new(),
            // create the symbol tables and add the global scope
            symbol_tables: Arc::new(RwLock::new(vec![HashMap::new()])),
        }
    }

    pub fn register_builtin(
        &mut self,
        fn_name: &str,
        fn_args: Vec<LiteralEnum>,
        fn_return_type: LiteralEnum,
        fn_pointer: BuiltinCallback,
    ) {

        // for the builtin functions we don't care of the variable name
        let mut typed_args = Vec::new();

        for el in fn_args.into_iter() {
            typed_args.push(TypedArg::new("".to_owned(), el));
        }

        let builtin = FunctionCall::new(typed_args, fn_return_type, FunctionKind::Builtin(fn_pointer));
        self.symbol_tables
            .write()
            .expect("Write lock already in use !")
            .first_mut()
            .unwrap()
            .insert(fn_name.to_owned(), Symbol::Function(builtin));
    }

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), String> {
        let contents = fs::read_to_string(file_path).expect("Can't open file !");

        self.exec(&contents)
    }

    pub fn exec(&mut self, code: &str) -> Result<(), String> {
        let tokens;
        let tree;

        match self.lexer.process(code) {
            Ok(t) => tokens = t,
            Err(e) => return Err(format!("Syntax error: {}", e)),
        }

        // println!("tokens: {:?}", &tokens);

        match self.parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(e) => return Err(format!("Parse error: {}", e)),
        }

        self.register_builtin("println", vec![LiteralEnum::Text(None)], LiteralEnum::Void, println);

        // add the global scope
        let mut symtables_unlocked = self
            .symbol_tables
            .write()
            .expect("Write lock already in use !");

        symtables_unlocked.push(HashMap::new());
        drop(symtables_unlocked);

        match tree.visit(self.symbol_tables.clone()) {
            Ok(None) => println!("Main function exited with no return value."),
            Ok(Some(x)) => println!("GOT {:?}", x),
            Err(e) => return Err(format!("Runtime error: {}", e)),
        }

        // println!("symbol tables: {:?}", self.symbol_tables);
        Ok(())
    }
}
