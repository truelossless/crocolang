use crate::builtin::*;
use crate::symbol::{SymbolContent, SymbolContent::*};
use crate::token::LiteralEnum::*;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![BuiltinFunction {
        name: "get".to_owned(),
        args: vec![Primitive(Str(None))],
        return_type: Primitive(Str(None)),
        pointer: get,
    }];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// returns the contents of a page given an url
fn get(mut args: Vec<Symbol>) -> SymbolContent {
    let url = get_arg_str(&mut args);

    let req = reqwest::blocking::get(&url);

    // return an empty string if we have an error
    // TODO: implement errors
    if req.is_err() {
        return Primitive(Str(Some(String::new())));
    }
    let res = req.unwrap();

    match res.text() {
        Ok(text) => Primitive(Str(Some(text))),
        Err(_) => Primitive(Str(Some(String::new()))),
    }
}
