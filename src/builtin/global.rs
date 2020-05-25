use crate::builtin::{BuiltinFunction, BuiltinModule};
use crate::token::LiteralEnum;

/// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        BuiltinFunction {
            name: "assert".to_owned(),
            args: vec![LiteralEnum::Bool(None)],
            return_type: LiteralEnum::Void,
            pointer: assert,
        },
        BuiltinFunction {
            name: "eprint".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: eprint,
        },
        BuiltinFunction {
            name: "eprintln".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: eprintln,
        },
        BuiltinFunction {
            name: "print".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: print,
        },
        BuiltinFunction {
            name: "println".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: println,
        },
    ];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// Exits if the first argument is false
fn assert(mut vars: Vec<LiteralEnum>) -> LiteralEnum{
    let arg = vars.remove(0).into_bool();

    if !arg {
        eprintln!("Assertion failed !");
        std::process::exit(1);
    }

    LiteralEnum::Void
}

/// Prints to stderr the first argument
fn eprint(mut vars: Vec<LiteralEnum>) -> LiteralEnum {
    let arg = vars.remove(0).into_str();

    eprint!("{}", arg);
    LiteralEnum::Void
}

/// Prints to stderr the first argument, with a line feed
fn eprintln(mut vars: Vec<LiteralEnum>) -> LiteralEnum {
    let arg = vars.remove(0).into_str();

    eprintln!("{}", arg);
    LiteralEnum::Void
}

/// Prints to stdout the first argument
fn print(mut vars: Vec<LiteralEnum>) -> LiteralEnum {
    let arg = vars.remove(0).into_str();

    print!("{}", arg);
    LiteralEnum::Void
}

/// Prints to stdout the first argument, with a line feed
fn println(mut vars: Vec<LiteralEnum>) -> LiteralEnum {
    let arg = vars.remove(0).into_str();

    println!("{}", arg);
    LiteralEnum::Void
}
