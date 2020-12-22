use crate::crocoi::stdlib::*;
use crate::crocoi::symbol::{ISymbol, ISymbol::*};
use crate::token::LiteralEnum::*;

use std::process::Command;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![BuiltinFunction {
        name: "exec".to_owned(),
        args: vec![SymbolType::Str],
        return_type: Some(SymbolType::Str),
        pointer: exec,
    }];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// executes a system command
fn exec(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let command_str = get_arg_str(&mut args);

    let command = if cfg!(windows) {
        Command::new("cmd").args(&["/C", &command_str]).output()
    } else {
        Command::new("sh").args(&["-c", &command_str]).output()
    };

    // whenenever an error happens, we're just returning empty strings for the moment
    // TODO: implement error types
    // fn exec() !str
    // !str is either ok(str) or err
    if let Ok(output) = command {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(Primitive(Str(stdout.into_owned())))
    } else {
        Some(Primitive(Str(String::new())))
    }
}
