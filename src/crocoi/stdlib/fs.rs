#![allow(clippy::unnecessary_wraps)]

use crate::crocoi::stdlib::*;
use crate::crocoi::symbol::{ISymbol, ISymbol::*};
use crate::symbol_type::SymbolType;
use crate::token::LiteralEnum::*;

use std::fs;
use std::path::Path;

// Module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        BuiltinFunction {
            name: "create_dir".to_owned(),
            args: vec![SymbolType::Str],
            return_type: None,
            pointer: create_dir,
        },
        BuiltinFunction {
            name: "exists".to_owned(),
            args: vec![SymbolType::Str],
            return_type: Some(SymbolType::Bool),
            pointer: exists,
        },
        BuiltinFunction {
            name: "read_file".to_owned(),
            args: vec![SymbolType::Str],
            return_type: Some(SymbolType::Str),
            pointer: read_file,
        },
        BuiltinFunction {
            name: "write_file".to_owned(),
            args: vec![SymbolType::Str, SymbolType::Str],
            return_type: None,
            pointer: write_file,
        },
    ];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// create a directory at <path>, as well as all the needed parent directories
fn create_dir(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let path = get_arg_str(&mut args);
    fs::create_dir_all(path).unwrap();
    None
}

/// retuns true if <path> exists
fn exists(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let path = get_arg_str(&mut args);
    Some(Primitive(Bool(Path::new(&path).exists())))
}

/// reads the content of the file at <path>
fn read_file(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let path = get_arg_str(&mut args);
    let contents = fs::read_to_string(path).unwrap_or_default();
    Some(Primitive(Str(contents)))
}

/// writes to <path> the <content> of a str
fn write_file(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let path = get_arg_str(&mut args);
    let content = get_arg_str(&mut args);
    fs::write(path, content).unwrap();
    None
}
