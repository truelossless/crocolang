use std::fs;

use crate::lexer::Lexer;
use crate::token::LiteralEnum;

use crate::parser::Parser;
use crate::parser::TypedArg;

use crate::ast::NodeResult;

use crate::symbol::{BuiltinCallback, FunctionCall, FunctionKind, SymTable, Symbol};

fn println(vars: Vec<LiteralEnum>) -> Result<LiteralEnum, String> {
    let arg = match &vars[0] {
        LiteralEnum::Text(Some(t)) => t,
        _ => unreachable!(),
    };

    println!("{}", arg);
    Ok(LiteralEnum::Void)
}

#[derive(Default)]
pub struct Interpreter {
    lexer: Lexer,
    parser: Parser,
    symtable: SymTable,
}

impl<'a> Interpreter {
    pub fn new() -> Self {
        Interpreter {
            parser: Parser::new(),
            lexer: Lexer::new(),
            // create the symbol tables and add the global scope
            symtable: SymTable::new(),
        }
    }

    pub fn register_builtin(
        &mut self,
        fn_name: &str,
        fn_args: Vec<LiteralEnum>,
        fn_return_type: LiteralEnum,
        fn_pointer: BuiltinCallback,
    ) -> Result<(), String> {
        // for the builtin functions we don't care of the variable name
        let mut typed_args = Vec::new();

        for el in fn_args.into_iter() {
            typed_args.push(TypedArg::new("".to_owned(), el));
        }

        let builtin = FunctionCall::new(
            typed_args,
            fn_return_type,
            FunctionKind::Builtin(fn_pointer),
        );
        self.symtable
            .register_fn(fn_name, Symbol::Function(builtin))?;
        Ok(())
    }

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), String> {
        let contents = fs::read_to_string(file_path).expect("Can't open file !");

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

        self.register_builtin(
            "println",
            vec![LiteralEnum::Text(None)],
            LiteralEnum::Void,
            println,
        )?;

        // add the global scope
        self.symtable.add_scope();

        match tree.visit(&mut self.symtable) {
            Ok(NodeResult::Literal(LiteralEnum::Void)) => {
                println!("Main function exited with no return value.")
            }
            Ok(x) => println!("Main function exited with {:?}", x),
            Err(e) => return Err(format!("Runtime error: {}", e)),
        }

        // println!("symbol tables: {:?}", self.symtable);
        Ok(())
    }
}
