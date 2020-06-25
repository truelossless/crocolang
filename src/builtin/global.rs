use crate::builtin::*;
use crate::symbol::{Symbol, Symbol::*};
use crate::token::LiteralEnum::*;

/// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        BuiltinFunction {
            name: "assert".to_owned(),
            args: vec![Primitive(Bool(None))],
            return_type: Primitive(Void),
            pointer: assert,
        },
        BuiltinFunction {
            name: "eprint".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Void),
            pointer: eprint,
        },
        BuiltinFunction {
            name: "eprintln".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Void),
            pointer: eprintln,
        },
        BuiltinFunction {
            name: "print".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Void),
            pointer: print,
        },
        BuiltinFunction {
            name: "println".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Void),
            pointer: println,
        },
    ];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// Exits if the first argument is false
fn assert(mut args: Vec<Symbol>) -> Symbol {
    let assertion = get_arg_bool(&mut args);

    if !assertion {
        eprintln!("Assertion failed !");
        std::process::exit(1);
    }

    Primitive(Void)
}

/// Prints to stderr the first argument
fn eprint(mut args: Vec<Symbol>) -> Symbol {
    let err = get_arg_str(&mut args);
    eprint!("{}", err);
    Primitive(Void)
}

/// Prints to stderr the first argument, with a line feed
fn eprintln(mut args: Vec<Symbol>) -> Symbol {
    let err = get_arg_str(&mut args);
    eprintln!("{}", err);
    Primitive(Void)
}

/// Prints to stdout the first argument
fn print(mut args: Vec<Symbol>) -> Symbol {
    let msg = get_arg_str(&mut args);
    print!("{}", msg);
    Primitive(Void)
}

/// Prints to stdout the first argument, with a line feed
fn println(mut args: Vec<Symbol>) -> Symbol {
    let msg = get_arg_str(&mut args);
    println!("{}", msg);
    Primitive(Void)
}
