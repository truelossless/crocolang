use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    types::IntType,
    types::StructType,
    values::{BasicValueEnum, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::token::CodePos;
use crate::{ast::NodeResult, symbol_type::SymbolType};
use crate::{error::CrocoError, symbol::SymTable};

// I'll be using a simple struct as in the README example for now
// https://github.com/TheDan64/inkwell/blob/master/README.md
/// A codegen unit
pub struct LCodegen<'ctx> {
    /// The llvm context
    pub context: &'ctx Context,
    /// The current module
    pub module: Module<'ctx>,
    /// The instruction builder
    pub builder: Builder<'ctx>,
    /// The symbol table containing pointers to variables
    pub symtable: LSymTable<'ctx>,
    /// The str type as defined in crocol
    pub str_type: StructType<'ctx>,
    /// The pointer size of this architecture
    pub ptr_size: IntType<'ctx>,
    /// The current function being built
    pub current_fn: Option<FunctionValue<'ctx>>,
    /// The pointer used as a return value in case of a sret function
    pub sret_ptr: Option<PointerValue<'ctx>>,
}

impl<'ctx> LCodegen<'ctx> {
    /// Creates a variable at the start of a block
    pub fn create_block_alloca(&self, ty: BasicTypeEnum<'ctx>, name: &str) -> PointerValue<'ctx> {
        // local variable
        if let Some(block) = self.builder.get_insert_block() {
            match block.get_first_instruction() {
                Some(first_instr) => self.builder.position_before(&first_instr),
                None => self.builder.position_at_end(block),
            }

            let alloca = self.builder.build_alloca(ty, name);
            self.builder.position_at_end(block);

            alloca

        // global variable
        } else {
            todo!("handle global variables !");
        }
    }

    pub fn alloc_str(&self, text: &str) -> PointerValue<'ctx> {
        // get the string as an llvm i8 array
        let char_array = self.context.const_string(text.as_bytes(), false);

        let alloca = self.create_block_alloca(self.str_type.into(), "allocastr");
        // since the size of the str is known we can directly malloc() the right amount
        let heap_ptr_ptr = self
            .builder
            .build_struct_gep(alloca, 0, "gepheapptr")
            .unwrap();
        let str_len = self.ptr_size.const_int(text.len() as u64, false);
        let new_heap_ptr = self
            .builder
            .build_array_malloc(self.context.i8_type(), str_len, "mallocstr")
            .unwrap();

        // bitcast the array into a pointer
        let char_array_ptr_type: BasicTypeEnum =
            char_array.get_type().ptr_type(AddressSpace::Generic).into();
        let char_ptr =
            self.builder
                .build_bitcast(new_heap_ptr, char_array_ptr_type, "bitcastarray");

        // store the string into the newly allocated memory
        self.builder
            .build_store(char_ptr.into_pointer_value(), char_array);

        // store into ptr the initialized ptr
        self.builder.build_store(heap_ptr_ptr, new_heap_ptr);

        // and finally update the len and max_len fields accordingly
        let len_ptr = self.builder.build_struct_gep(alloca, 1, "geplen").unwrap();
        self.builder.build_store(len_ptr, str_len);
        let max_len_ptr = self
            .builder
            .build_struct_gep(alloca, 2, "gepmaxlen")
            .unwrap();
        self.builder.build_store(max_len_ptr, str_len);

        alloca
    }

    /// Dereferences a pointer if needed, or returns the corresponding enum
    pub fn auto_deref(&self, value: BasicValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match value {
            BasicValueEnum::PointerValue(p) => self.builder.build_load(p, "loadautoderef"),
            value => value,
        }
    }
}

// we have to keep track of the symbol type.
/// A symbol in the crocol backend
#[derive(Clone, Debug)]
pub struct LSymbol<'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub symbol_type: SymbolType,
}

impl<'ctx> LSymbol<'ctx> {
    pub fn into_bool(self) -> Result<IntValue<'ctx>, String> {
        match self.symbol_type {
            SymbolType::Bool => Ok(self.value.into_int_value()),
            _ => Err("expected a bool".to_owned()),
        }
    }
}

/// The result returned by a node.  
/// A symbol value is a LSymbol.  
/// A symbol in the symtable is also a LSymbol but the value is always guarenteed a pointer.
pub type LNodeResult<'ctx> = NodeResult<LSymbol<'ctx>, LSymbol<'ctx>>;

/// The symtable with the crocol-specific types.
pub type LSymTable<'ctx> = SymTable<LSymbol<'ctx>>;

impl<'ctx> LNodeResult<'ctx> {
    /// Loads a variable, or return the value directly
    pub fn into_symbol(
        self,
        codegen: &LCodegen<'ctx>,
        code_pos: &CodePos,
    ) -> Result<LSymbol<'ctx>, CrocoError> {
        match self {
            LNodeResult::Variable(var) => {
                let value = codegen
                    .builder
                    .build_load(var.value.into_pointer_value(), "loadvar");
                Ok(LSymbol {
                    value,
                    symbol_type: var.symbol_type,
                })
            }
            LNodeResult::Value(val) => Ok(val),
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }

    /// Returns a variable, or store a value to get a PointerValue
    pub fn into_pointer(
        self,
        codegen: &LCodegen<'ctx>,
        code_pos: &CodePos,
    ) -> Result<PointerValue<'ctx>, CrocoError> {
        match self {
            LNodeResult::Variable(var) => Ok(var.value.into_pointer_value()),
            LNodeResult::Value(val) => {
                Ok(codegen.create_block_alloca(val.value.get_type(), "storeval"))
            }
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }
}
