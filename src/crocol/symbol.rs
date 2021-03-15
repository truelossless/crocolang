use inkwell::{
    attributes::{Attribute, AttributeLoc},
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    types::IntType,
    types::StructType,
    values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::crocol::utils::get_or_define_function;
use crate::{ast::BackendNode, token::CodePos};
use crate::{ast::NodeResult, symbol_type::SymbolType};
use crate::{error::CrocoError, symbol::SymTable};

use super::utils::{get_llvm_type, get_or_define_struct};

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
    /// The array type as defined in crocol
    pub array_type: StructType<'ctx>,
    /// The str type as defined in crocol
    pub str_type: StructType<'ctx>,
    /// The pointer size of this architecture
    pub ptr_size: IntType<'ctx>,
    /// The current function being built
    pub current_fn: Option<FunctionValue<'ctx>>,
    /// The current block when a loop is being built. This is used for the continue instruction.
    pub current_loop_block: Option<BasicBlock<'ctx>>,
    /// The current block when a loop is being built. This is used for the continue instruction.
    pub current_loop_end_block: Option<BasicBlock<'ctx>>,
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

    /// Builds a function from an AST node
    pub fn build_function(
        &mut self,
        fn_name: &str,
        mut fn_body: Box<dyn BackendNode>,
        code_pos: &CodePos,
    ) -> Result<(), CrocoError> {
        let fn_decl = self.symtable.get_function_decl(fn_name).unwrap().clone();

        // we're done with the current variables
        self.symtable.pop_symbols();

        let function = get_or_define_function(fn_name, &fn_decl, self);
        let sret_fn = matches!(
            fn_decl.return_type,
            Some(SymbolType::Struct(_)) | Some(SymbolType::Str)
        );

        // add the sret tag to the first param if needed
        if sret_fn {
            self.sret_ptr = Some(function.get_first_param().unwrap().into_pointer_value());
            let sret_num = Attribute::get_named_enum_kind_id("sret");
            let sret_attr = self.context.create_enum_attribute(sret_num, 0);
            function.add_attribute(AttributeLoc::Param(0), sret_attr);
        } else {
            self.sret_ptr = None
        }

        // the start of the function
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        // in case of a function returning a struct, skip the first param which is the return value
        let args_iter = if sret_fn {
            fn_decl.args.iter().zip(function.get_param_iter().skip(1))
        } else {
            // skip 0 so both iters have the same type
            fn_decl.args.iter().zip(function.get_param_iter().skip(0))
        };

        self.current_fn = Some(function);

        // inject the function arguments in the body
        for (arg, param_value) in args_iter {
            // to comply with the "C ABI", fn(Struct a) is changed to fn(&Struct a)
            // this means we need to add a memcpy in the function body.
            let abi_ptr = match &arg.arg_type {
                // TODO: copy correctly heap allocated memory
                SymbolType::Str | SymbolType::Array(_) | SymbolType::Struct(_) => {
                    let ty = if let SymbolType::Struct(struct_name) = &arg.arg_type {
                        let struct_ty = self
                            .symtable
                            .get_struct_decl(struct_name)
                            .map_err(|e| CrocoError::new(code_pos, e))?;
                        get_or_define_struct(struct_name, &struct_ty, self)
                    } else {
                        self.str_type
                    };

                    let copy_alloca = self.create_block_alloca(ty.into(), "copy");

                    self.builder
                        .build_memcpy(
                            copy_alloca,
                            8,
                            param_value.into_pointer_value(),
                            8,
                            ty.size_of().unwrap(),
                        )
                        .unwrap();
                    copy_alloca
                }
                SymbolType::Bool | SymbolType::Num | SymbolType::Fnum | SymbolType::Ref(_) => {
                    let param_ptr = self.create_block_alloca(param_value.get_type(), "param");
                    self.builder.build_store(param_ptr, param_value);
                    param_ptr
                }
                _ => unimplemented!(),
            };

            let symbol = LSymbol {
                symbol_type: arg.arg_type.clone(),
                value: abi_ptr.into(),
            };
            self.symtable
                .insert_symbol(&arg.arg_name, symbol)
                .map_err(|e| CrocoError::new(&code_pos, e))?;
        }

        // populate the function body
        let ret_val = match fn_body.crocol(self)? {
            LNodeResult::Return(ret) => ret,
            LNodeResult::Break => return Err(CrocoError::break_in_function_error(code_pos)),
            LNodeResult::Continue => return Err(CrocoError::continue_in_function_error(code_pos)),
            LNodeResult::Value(val) => Some(val),
            LNodeResult::Variable(var) => Some(var),
            LNodeResult::Void => None,
        };

        // make sure the return value matches the function declaration
        let ret_ty_opt = ret_val.map(|x| x.symbol_type);
        match (&fn_decl.return_type, &ret_ty_opt) {
            (None, None) => (),
            (Some(fn_ty), Some(ret_ty)) if fn_ty != ret_ty => {
                return Err(CrocoError::wrong_return(
                    fn_decl.return_type.as_ref(),
                    ret_ty_opt.as_ref(),
                    code_pos,
                ))
            }
            (Some(_), Some(_)) => (),
            _ => {
                return Err(CrocoError::wrong_return(
                    fn_decl.return_type.as_ref(),
                    ret_ty_opt.as_ref(),
                    code_pos,
                ))
            }
        }

        Ok(())
    }

    /// Allocates a CrocoArray given specific elements.
    pub fn alloc_array(&self, elements: Vec<LSymbol>) -> PointerValue<'ctx> {
        let alloca = self.create_block_alloca(self.array_type.into(), "array");

        // calls to alloc_array are always guaranteed to have at least one element in the array so this is fine
        let el_llvm_ty = get_llvm_type(&elements[0].symbol_type, self);

        let heap_ptr_ptr = self
            .builder
            .build_struct_gep(alloca, 0, "arrayheapptr")
            .unwrap();
        let array_len = self.ptr_size.const_int(elements.len() as u64, false);
        let new_heap_ptr = self
            .builder
            .build_array_malloc(el_llvm_ty, array_len, "mallocarr")
            .unwrap();

        // store the elements into the newly allocated memory
        // note that this requires a stack allocation before the elements are copied
        // on the heap, but with our current design it is hard to do it differently.
        // hopefully llvm is able to optimize away the unneeded allocations.
        for (i, el) in elements.into_iter().enumerate() {
            let index = self.ptr_size.const_int(i as u64, false);

            // SAFETY: the pointer is allocated with the length of all elements,
            // so the GEP should always be in bounds.
            let single_el_ptr = unsafe {
                self.builder
                    .build_gep(new_heap_ptr, &[index], "arrayelement")
            };

            self.builder.build_store(single_el_ptr, el.value);
        }

        // store into ptr the initialized ptr
        let void_ptr = self.builder.build_bitcast(
            new_heap_ptr,
            self.context.i8_type().ptr_type(AddressSpace::Generic),
            "voidptr",
        );
        self.builder.build_store(heap_ptr_ptr, void_ptr);

        // and finally update the len and max_len fields accordingly
        let len_ptr = self
            .builder
            .build_struct_gep(alloca, 1, "arraylen")
            .unwrap();
        self.builder.build_store(len_ptr, array_len);
        let max_len_ptr = self
            .builder
            .build_struct_gep(alloca, 2, "arraymaxlen")
            .unwrap();
        self.builder.build_store(max_len_ptr, array_len);

        alloca
    }

    /// Allocates a CrocStr given a specific text
    pub fn alloc_str(&self, text: &str) -> PointerValue<'ctx> {
        // get the string as an llvm i8 array
        let char_array = self.context.const_string(text.as_bytes(), false);

        let alloca = self.create_block_alloca(self.str_type.into(), "str");
        // since the size of the str is known we can directly malloc() the right amount
        let heap_ptr_ptr = self
            .builder
            .build_struct_gep(alloca, 0, "strheapptr")
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
        let len_ptr = self.builder.build_struct_gep(alloca, 1, "strlen").unwrap();
        self.builder.build_store(len_ptr, str_len);
        let max_len_ptr = self
            .builder
            .build_struct_gep(alloca, 2, "strmaxlen")
            .unwrap();
        self.builder.build_store(max_len_ptr, str_len);

        alloca
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
    pub fn into_bool(self, code_pos: &CodePos) -> Result<IntValue<'ctx>, CrocoError> {
        match self.symbol_type {
            SymbolType::Bool => Ok(self.value.into_int_value()),
            _ => Err(CrocoError::new(code_pos, "expected a bool")),
        }
    }

    pub fn into_fnum(self, code_pos: &CodePos) -> Result<FloatValue<'ctx>, CrocoError> {
        match self.symbol_type {
            SymbolType::Fnum => Ok(self.value.into_float_value()),
            _ => Err(CrocoError::new(code_pos, "expected a fnum")),
        }
    }

    pub fn into_num(self, code_pos: &CodePos) -> Result<IntValue<'ctx>, CrocoError> {
        match self.symbol_type {
            SymbolType::Num => Ok(self.value.into_int_value()),
            _ => Err(CrocoError::new(code_pos, "expected a num")),
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
    ) -> Result<LSymbol<'ctx>, CrocoError> {
        match self {
            LNodeResult::Variable(var) => Ok(var),
            LNodeResult::Value(val) => {
                let alloca = codegen.create_block_alloca(val.value.get_type(), "storeval");
                codegen.builder.build_store(alloca, val.value);

                Ok(LSymbol {
                    value: alloca.into(),
                    symbol_type: val.symbol_type,
                })
            }
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }

    /// Gets the symbol type of possible
    pub fn get_symbol_type(&self, code_pos: &CodePos) -> Result<&SymbolType, CrocoError> {
        match self {
            LNodeResult::Variable(LSymbol { symbol_type, .. })
            | LNodeResult::Value(LSymbol { symbol_type, .. }) => Ok(symbol_type),
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }
}
