use std::{cell::RefCell, rc::Rc};

use unicode_segmentation::UnicodeSegmentation;

use crate::crocoi::symbol::ISymbol;
use crate::token::LiteralEnum::*;
use crate::{builtin::*, crocoi::symbol::Array};

/// module definition
pub fn get_module() -> BuiltinModule {
    let functions = vec![
        // global methods
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
        // array methods
        // since we don't have generics we can't yet implement push() and pop() :S
        BuiltinFunction {
            name: "_array_join".to_owned(),
            args: vec![SymbolType::Str],
            return_type: Some(SymbolType::Str),
            pointer: _array_join
        },

        BuiltinFunction {
            name: "_array_len".to_owned(),
            args: Vec::new(),
            return_type: Some(SymbolType::Num),
            pointer: _array_len,
        },
        // num methods
        BuiltinFunction {
            name: "_num_times".to_owned(),
            args: vec![SymbolType::Num],
            return_type: Some(SymbolType::Array(Box::new(SymbolType::Num))),
            pointer: _num_times,
        },
        // str methods
        BuiltinFunction {
            name: "_str_len".to_owned(),
            args: Vec::new(),
            return_type: Some(SymbolType::Num),
            pointer: _str_len,
        },
        BuiltinFunction {
            name: "_str_slice".to_owned(),
            args: vec![SymbolType::Num, SymbolType::Num],
            return_type: Some(SymbolType::Str),
            pointer: _str_slice,
        },
        BuiltinFunction {
            name: "_str_split".to_owned(),
            args: vec![SymbolType::Str],
            return_type: Some(SymbolType::Array(Box::new(SymbolType::Str))),
            pointer: _str_split,
        },
        BuiltinFunction {
            name: "_str_trim".to_owned(),
            args: Vec::new(),
            return_type: Some(SymbolType::Str),
            pointer: _str_trim,
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
    let msg = dbg!(get_arg_str(&mut args));
    println!("{}", msg);
    None
}

/// Array methods

/// Joins a str array into a str
// TODO checks that the array is an array of str
fn _array_join(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let array = get_arg_array(&mut args);
    let delimiter = get_arg_str(&mut args);
    let string = array
        .contents
        .iter()
        .map(|x| {
            x.borrow()
                .clone()
                .into_primitive()
                .unwrap()
                .into_str()
                .unwrap()
        })
        .collect::<Vec<_>>()
        .join(&delimiter);

    Some(ISymbol::Primitive(Str(string)))
}

/// Returns the length of an array
fn _array_len(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let array = get_arg_array(&mut args);
    Some(ISymbol::Primitive(Num(array.contents.len() as f32)))
}

/// Num methods

/// Returns an array containing `times` times the number
fn _num_times(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let num = get_arg_num(&mut args);
    let times = get_arg_num(&mut args) as usize;

    let mut arr = Vec::with_capacity(times);
    for _ in 0..times {
        arr.push(Rc::new(RefCell::new(ISymbol::Primitive(Num(num)))));
    }

    Some(ISymbol::Array(Array {
        contents: arr,
        array_type: Box::new(SymbolType::Num),
    }))
}

/// Str methods

/// Returns the length of a str
fn _str_len(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let string = get_arg_str(&mut args);
    Some(ISymbol::Primitive(Num(
        string.graphemes(true).count() as f32
    )))
}

/// Slice a str given a start offset and an end offset.  
/// similar to JavaScript String.prototype.slice  
/// @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice
fn _str_slice(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let string = get_arg_str(&mut args);
    let mut start = get_arg_num(&mut args) as isize;
    let mut end = get_arg_num(&mut args) as isize;

    let graphemes = string.graphemes(true);

    if start < 0 || end < 0 {
        let len = graphemes.clone().count() as isize;

        if start < 0 {
            start += len;
        }

        if end < 0 {
            end += len;
        }
    }

    let mut len = end - start;
    if len < 0 {
        len = 0;
    }

    let ret: String = graphemes.skip(start as usize).take(len as usize).collect();
    Some(ISymbol::Primitive(Str(ret)))
}

/// Returns a str array splitted with the first arg as the separator
fn _str_split(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    let string = get_arg_str(&mut args);
    let delimiter = get_arg_str(&mut args);
    let arr = string
        .split(&delimiter)
        .map(|x| Rc::new(RefCell::new(ISymbol::Primitive(Str(x.to_owned())))))
        .collect();
    Some(ISymbol::Array(Array {
        array_type: Box::new(SymbolType::Str),
        contents: arr,
    }))
}

/// Returns the trimmed string
fn _str_trim(mut args: Vec<ISymbol>) -> Option<ISymbol> {
    Some(ISymbol::Primitive(Str(get_arg_str(&mut args)
        .trim()
        .to_owned())))
}
