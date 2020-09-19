use std::{cell::RefCell, convert::TryInto};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    types::IntType,
    values::BasicValueEnum,
    values::{AnyValueEnum, FunctionValue, PointerValue},
types::StructType};

use crate::{
    symbol::{SymTable, Symbol},
    symbol_type::SymbolType,
};

// I'll be using a simple struct as in the README example for now
// https://github.com/TheDan64/inkwell/blob/master/README.md
/// a codegen unit
pub struct Codegen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub symtable: RefCell<SymTable<LSymbol<'ctx>>>,
    pub str_type: StructType<'ctx>,
    pub ptr_size: IntType<'ctx>, // this platform's isize width
    pub current_fn: RefCell<FunctionValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn create_entry_block_alloca(
        &self,
        ty: BasicTypeEnum<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let entry = self.current_fn.borrow().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => self.builder.position_before(&first_instr),
            None => self.builder.position_at_end(entry),
        }

        self.builder.build_alloca(ty, name)
    }

    pub fn get_ptr_value(&self, any_value: AnyValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        self.builder.build_load(any_value.into_pointer_value(), "load")
    }

    /// dereferences a pointer if needed, or returns the corresponding enum
    pub fn auto_deref(&self, value: AnyValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match value {
            AnyValueEnum::PointerValue(p) => self.builder.build_load(p, "auto deref load"),
            value => value.try_into().unwrap()
        }
    }
}

// we have to keep track of the symbol type.
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
