use crate::builtin::*;
use crate::symbol::{Symbol, SymbolContent::*};
use crate::token::LiteralEnum::*;

use std::process::Command;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![BuiltinFunction {
        name: "exec".to_owned(),
        args: vec![Primitive(Str(None))],
        return_type: Primitive(Str(None)),
        pointer: exec,
    }];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// executes a system command
fn exec(mut args: Vec<Symbol>) -> SymbolContent {
    let command_str = get_arg_str(&mut args);

    let command = if cfg!(target_os = "windows") {
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
        Primitive(Str(Some(stdout.into_owned())))
    } else {
        Primitive(Str(Some(String::new())))
    }
}
