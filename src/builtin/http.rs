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

    let res = ureq::get(&url).call().into_string();

    // return an empty string if we have an error
    // TODO: implement errors
    if let Ok(text) = res {
        Primitive(Str(Some(text)))
    } else {
        Primitive(Str(Some(String::new())))
    }
}
