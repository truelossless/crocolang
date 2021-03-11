#![allow(clippy::unnecessary_wraps)]

use crate::crocoi::stdlib::*;
use crate::crocoi::symbol::{ISymbol, ISymbol::*};
use crate::token::LiteralEnum::*;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![BuiltinFunction {
        name: "get".to_owned(),
        args: vec![SymbolType::Str],
        return_type: Some(SymbolType::Str),
        pointer: get,
    }];

    let vars = Vec::new();

    BuiltinModule { functions, vars }
}

/// returns the contents of a page given an url
fn get(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let url = get_arg_str(&mut args);

    let res = ureq::get(&url).call().into_string();

    // return an empty string if we have an error
    // TODO: implement errors
    if let Ok(text) = res {
        Some(Primitive(Str(text)))
    } else {
        Some(Primitive(Str(String::new())))
    }
}
