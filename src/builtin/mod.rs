// all the croco builtin functions live here.

// the filesystem module
pub mod fs;

// the global module, which is automatically brought to scope
pub mod global;

// the http module
pub mod http;

// the math module
pub mod math;

// the os module
pub mod os;

use crate::token::LiteralEnum;

/// callback to a built-in function
pub type BuiltinCallback = fn(Vec<LiteralEnum>) -> LiteralEnum;

/// representation of a built-in function
pub struct BuiltinFunction {
    pub name: String,
    pub args: Vec<LiteralEnum>,
    pub return_type: LiteralEnum,
    pub pointer: BuiltinCallback
}

pub struct BuiltinVar {
    pub name: String,
    pub value: LiteralEnum
}

/// representation of a built-in module
pub struct BuiltinModule {
    pub functions: Vec<BuiltinFunction>,
    pub vars: Vec<BuiltinVar>
}

// since there is no global state in Rust,
// I'm creating some sort of "module manager",
// which defines all needed modules.

/// retreive a built-in module by name
pub fn get_module(name: &str) -> Option<BuiltinModule> {

    let module = match name {
        "fs" => fs::get_module(),
        "global" => global::get_module(),
        "http" => http::get_module(),
        "math" => math::get_module(),
        "os" => os::get_module(),
        _ => return None
    };

    Some(module)
}