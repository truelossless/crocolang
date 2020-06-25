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

use crate::symbol::{Symbol, Struct};

/// callback to a built-in function
pub type BuiltinCallback = fn(Vec<Symbol>) -> Symbol;

/// representation of a built-in function
pub struct BuiltinFunction {
    pub name: String,
    pub args: Vec<Symbol>,
    pub return_type: Symbol,
    pub pointer: BuiltinCallback
}

pub struct BuiltinVar {
    pub name: String,
    pub value: Symbol
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

// utils to easily get args
pub fn get_arg_str(args: &mut Vec<Symbol>) -> String {
    args.remove(0).into_primitive().unwrap().into_str()
}

pub fn _get_arg_num(args: &mut Vec<Symbol>) -> f32 {
    args.remove(0).into_primitive().unwrap().into_num()
}

pub fn get_arg_bool(args: &mut Vec<Symbol>) -> bool {
    args.remove(0).into_primitive().unwrap().into_bool()
}

pub fn _get_arg_struct(args: &mut Vec<Symbol>) -> Struct {
    args.remove(0).into_struct().unwrap()
}