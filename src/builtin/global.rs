use crate::builtin::*;
use crate::crocoi::symbol::ISymbol;

/// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        BuiltinFunction {
            name: "assert".to_owned(),
            args: vec![SymbolType::Bool],
            return_type: None,
            pointer: assert,
        },
        BuiltinFunction {
            name: "eprint".to_owned(),
            args: vec![SymbolType::Str],
            return_type: None,
            pointer: eprint,
        },
        BuiltinFunction {
            name: "eprintln".to_owned(),
            args: vec![SymbolType::Str],
            return_type: None,
            pointer: eprintln,
        },
        BuiltinFunction {
            name: "print".to_owned(),
            args: vec![SymbolType::Str],
            return_type: None,
            pointer: print,
        },
        BuiltinFunction {
            name: "println".to_owned(),
            args: vec![SymbolType::Str],
            return_type: None,
            pointer: println,
        },
    ];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// Exits if the first argument is false
fn assert(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let assertion = get_arg_bool(&mut args);

    if !assertion {
        eprintln!("Assertion failed !");
        std::process::exit(1);
    }
    None
}

/// Prints to stderr the first argument
fn eprint(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let err = get_arg_str(&mut args);
    eprint!("{}", err);
    None
}

/// Prints to stderr the first argument, with a line feed
fn eprintln(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let err = get_arg_str(&mut args);
    eprintln!("{}", err);
    None
}

/// Prints to stdout the first argument
fn print(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let msg = get_arg_str(&mut args);
    print!("{}", msg);
    None
}

/// Prints to stdout the first argument, with a line feed
fn println(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let msg = get_arg_str(&mut args);
    println!("{}", msg);
    None
}
