use crate::builtin::{BuiltinFunction, BuiltinModule};
use crate::token::LiteralEnum;

use reqwest;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![BuiltinFunction {
        name: "get".to_owned(),
        args: vec![LiteralEnum::Str(None)],
        return_type: LiteralEnum::Str(None),
        pointer: get,
    }];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// returns the contents of a page given an url
fn get(mut args: Vec<LiteralEnum>) -> LiteralEnum {
    let arg = args.remove(0).into_str();

    let req = reqwest::blocking::get(&arg);
    
    // return an empty string if we have an error
    // TODO: implement errors
    if req.is_err() {
        return LiteralEnum::Str(Some(String::new()));
    }
    let res = req.unwrap();

    match res.text() {
        Ok(text) => LiteralEnum::Str(Some(text)),
        Err(_) => LiteralEnum::Str(Some(String::new()))
    }
}
