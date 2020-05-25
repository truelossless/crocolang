use crate::builtin::{BuiltinFunction, BuiltinModule};
use crate::token::LiteralEnum;

use std::process::Command;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![BuiltinFunction {
        name: "exec".to_owned(),
        args: vec![LiteralEnum::Str(None)],
        return_type: LiteralEnum::Str(None),
        pointer: exec,
    }];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// executes a system command
fn exec(mut args: Vec<LiteralEnum>) -> LiteralEnum {
    let arg = args.remove(0).into_str();
    
    let command = if cfg!(target_os = "windows") {
        Command::new("cmd")
        .args(&["/C", &arg])
        .output()
    } else {
        Command::new("sh")
        .args(&["-c", &arg])
        .output()
    };
    
    // whenenever an error happens, we're just returning empty strings for the moment
    // TODO: implement error types
    // fn exec() !str
    // !str is either ok(str) or err
    if let Ok(output) = command 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        LiteralEnum::Str(Some(stdout.into_owned()))
    } else {
        LiteralEnum::Str(Some("".to_owned()))
    }
}
