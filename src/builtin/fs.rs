use crate::builtin::*;
use crate::symbol::{Symbol, Symbol::*};
use crate::token::LiteralEnum::*;

use std::fs;
use std::path::Path;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        BuiltinFunction {
            name: "create_dir".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Void),
            pointer: create_dir,
        },
        BuiltinFunction {
            name: "exists".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Bool(None)),
            pointer: exists,
        },
        BuiltinFunction {
            name: "read_file".to_owned(),
            args: vec![Primitive(Str(None))],
            return_type: Primitive(Str(None)),
            pointer: read_file,
        },
        BuiltinFunction {
            name: "write_file".to_owned(),
            args: vec![Primitive(Str(None)), Primitive(Str(None))],
            return_type: Primitive(Void),
            pointer: write_file,
        },
    ];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// create a directory at <path>, as well as all the needed parent directories
fn create_dir(mut args: Vec<Symbol>) -> Symbol {
    let path = get_arg_str(&mut args);
    fs::create_dir_all(path).unwrap();
    Primitive(Void)
}

/// retuns true if <path> exists
fn exists(mut args: Vec<Symbol>) -> Symbol {
    let path = get_arg_str(&mut args);
    Primitive(Bool(Some(Path::new(&path).exists())))
}

/// reads the content of the file at <path>
fn read_file(mut args: Vec<Symbol>) -> Symbol {
    let path = get_arg_str(&mut args);
    let contents = fs::read_to_string(path).unwrap_or_default();
    Primitive(Str(Some(contents)))
}

/// writes to <path> the <content> of a str
fn write_file(mut args: Vec<Symbol>) -> Symbol {
    let path = get_arg_str(&mut args);
    let content = get_arg_str(&mut args);
    fs::write(path, content).unwrap();
    Primitive(Void)
}
