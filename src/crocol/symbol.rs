use std::cell::RefCell;

use inkwell::{
    context::Context,
    module::Module,
    types::IntType,
    values::{AnyValueEnum, FunctionValue, PointerValue}, builder::Builder,
};

use crate::{symbol::{Symbol, SymTable}, symbol_type::SymbolType};

// I'll be using a simple struct as in the README example for now
// https://github.com/TheDan64/inkwell/blob/master/README.md
/// a codegen unit
pub struct Codegen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub symtable: RefCell<SymTable<LSymbol<'ctx>>>,
    pub ptr_size: IntType<'ctx>, // this platform's isize width
    pub current_fn: RefCell<FunctionValue<'ctx>>,
}

// since PointerValues are just like C's void pointers, we have to keep track of the symbol type.
/// a symbol in the crocol backend
#[derive(Clone)]
pub struct LSymbol<'ctx> {
    pub pointer: PointerValue<'ctx>,
    pub symbol_type: SymbolType,
}

impl Symbol for LSymbol<'_> {

    fn to_type(&self) -> SymbolType {
        self.symbol_type.clone()
    }
}

/// The type of value returned by a node in the crocol backend
#[derive(Clone)]
pub enum LNodeResult<'ctx> {
    /// a break statement
    Break,
    /// a continue statement
    Continue,
    /// a return statement
    /// e.g return 3
    Return(AnyValueEnum<'ctx>),
    /// a symbol
    /// e.g a struct or a primitive
    Symbol(AnyValueEnum<'ctx>),
    // void values doesn't exist in llvm, here is ours
    /// a void value
    Void,
}

impl<'ctx> LNodeResult<'ctx> {
    pub fn into_symbol(self) -> AnyValueEnum<'ctx> {
        match self {
            Self::Symbol(s) => s,
            _ => unreachable!(),
        }
    }
}
