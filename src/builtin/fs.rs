use crate::builtin::{BuiltinFunction, BuiltinModule};
use crate::token::LiteralEnum;

use std::fs;
use std::path::Path;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        BuiltinFunction {
            name: "create_dir".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: create_dir,
        },
        BuiltinFunction {
            name: "exists".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: exists,
        },
        BuiltinFunction {
            name: "read_file".to_owned(),
            args: vec![LiteralEnum::Str(None)],
            return_type: LiteralEnum::Str(None),
            pointer: read_file,
        },
        BuiltinFunction {
            name: "write_file".to_owned(),
            args: vec![LiteralEnum::Str(None), LiteralEnum::Str(None)],
            return_type: LiteralEnum::Void,
            pointer: write_file,
        },
    ];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// create a directory at <path>, as well as all the needed parent directories
fn create_dir(mut args: Vec<LiteralEnum>) -> LiteralEnum {
    let path = args.remove(0).into_str();
    let _ = fs::create_dir_all(path);
    LiteralEnum::Void
}

/// retuns true if <path> exists
fn exists(mut args: Vec<LiteralEnum>) -> LiteralEnum {
    let path = args.remove(0).into_str();
    LiteralEnum::Bool(Some(Path::new(&path).exists()))
}

/// reads the content of the file at <path>
fn read_file(mut args: Vec<LiteralEnum>) -> LiteralEnum {
    let path = args.remove(0).into_str();
    let contents = fs::read_to_string(path).unwrap_or_default();
    LiteralEnum::Str(Some(contents))
}

/// writes to <path> the <content> of a str
fn write_file(mut args: Vec<LiteralEnum>) -> LiteralEnum {
    let path = args.remove(0).into_str();
    let content = args.remove(0).into_str();
    let _ = fs::write(path, content);
    LiteralEnum::Void
}
