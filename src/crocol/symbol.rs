use std::convert::TryInto;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    types::IntType,
    types::StructType,
    values::BasicValueEnum,
    values::{AnyValueEnum, FunctionValue, PointerValue},
};

use crate::{ast::NodeResult, symbol::SymTable, symbol_type::SymbolType};

// I'll be using a simple struct as in the README example for now
// https://github.com/TheDan64/inkwell/blob/master/README.md
/// a codegen unit
pub struct Codegen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub symtable: SymTable<LSymbol<'ctx>>,
    pub str_type: StructType<'ctx>,
    pub ptr_size: IntType<'ctx>, // this platform's isize width
    pub current_fn: FunctionValue<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    // create a variable at the start of a block
    pub fn create_entry_block_alloca(
        &self,
        ty: BasicTypeEnum<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let entry = self.current_fn.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => self.builder.position_before(&first_instr),
            None => self.builder.position_at_end(entry),
        }

        let alloca = self.builder.build_alloca(ty, name);
        self.builder.position_at_end(entry);

        alloca
    }

    pub fn get_ptr_value(&self, any_value: AnyValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        self.builder
            .build_load(any_value.into_pointer_value(), "load")
    }

    /// dereferences a pointer if needed, or returns the corresponding enum
    pub fn auto_deref(&self, value: AnyValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match value {
            AnyValueEnum::PointerValue(p) => self.builder.build_load(p, "loadautoderef"),
            value => value.try_into().unwrap(),
        }
    }
}

// we have to keep track of the symbol type.
/// a symbol in the crocol backend
#[derive(Clone)]
pub struct LSymbol<'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub symbol_type: SymbolType,
}

/// The result returned by a node.  
/// A symbol value is a LSymbol.  
/// A symbol in the symtable is also a LSymbol but the value is always guarenteed a pointer.
pub type LNodeResult<'ctx> = NodeResult<LSymbol<'ctx>, LSymbol<'ctx>>;
