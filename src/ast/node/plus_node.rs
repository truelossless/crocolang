#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::ISymbol, utils::get_value, INodeResult, ISymTable};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult, LSymbol};

use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::symbol_type::SymbolType;
use crate::token::{CodePos, LiteralEnum::*};

/// a node used for addition and concatenation
#[derive(Clone)]
pub struct PlusNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl PlusNode {
    pub fn new(code_pos: CodePos) -> Self {
        PlusNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

/// node handling additions and concatenations
impl AstNode for PlusNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, symtable, &self.code_pos)?;
        let right_val = get_value(&mut self.right, symtable, &self.code_pos)?;

        // different kinds of additions can happen (concatenation or number addition)
        // the PlusNode also works for concatenation.
        let value = match (left_val, right_val) {
            (Num(n1), Num(n2)) => Num(n1 + n2),
            (Str(s1), Str(s2)) => Str(format!("{}{}", s1, s2)),
            _ => return Err(CrocoError::add_error(&self.code_pos)),
        };
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let left_val = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;
        let right_val = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        match (left_val.symbol_type, right_val.symbol_type) {
            (SymbolType::Num, SymbolType::Num) => {
                let left_float = left_val.value.into_float_value();
                let right_float = right_val.value.into_float_value();

                let res = codegen
                    .builder
                    .build_float_add(left_float, right_float, "tmpadd");

                Ok(LNodeResult::Value(LSymbol {
                    value: res.into(),
                    symbol_type: SymbolType::Num,
                }))
            }

            (SymbolType::Str, SymbolType::Str) => {
                let left_str = left_val.value.into_pointer_value();
                let left_heap_ptr_ptr = codegen
                    .builder
                    .build_struct_gep(left_str, 0, "gepheapptr")
                    .unwrap();
                let left_heap_ptr = codegen.builder.build_load(left_heap_ptr_ptr, "loadheapptr");
                let left_len_ptr = codegen
                    .builder
                    .build_struct_gep(left_str, 1, "geplen")
                    .unwrap();
                let left_len = codegen.builder.build_load(left_len_ptr, "loadlen");

                let right_str = right_val.value.into_pointer_value();
                let right_heap_ptr_ptr = codegen
                    .builder
                    .build_struct_gep(right_str, 0, "gepheapptr")
                    .unwrap();
                let right_heap_ptr = codegen
                    .builder
                    .build_load(right_heap_ptr_ptr, "loadheapptr");
                let right_len_ptr = codegen
                    .builder
                    .build_struct_gep(right_str, 1, "geplen")
                    .unwrap();
                let right_len = codegen.builder.build_load(right_len_ptr, "loadlen");

                // get the combined length of both strings
                let combined_length = codegen.builder.build_int_add(
                    left_len.into_int_value(),
                    right_len.into_int_value(),
                    "addlen",
                );

                // create our new string
                let str_type = codegen.module.get_struct_type("struct.CrocoStr").unwrap();
                let alloca = codegen.create_entry_block_alloca(str_type.into(), "allocastr");

                let new_len_ptr = codegen
                    .builder
                    .build_struct_gep(alloca, 1, "geplen")
                    .unwrap();
                codegen.builder.build_store(new_len_ptr, combined_length);

                let new_max_len_ptr = codegen
                    .builder
                    .build_struct_gep(alloca, 2, "gepmaxlen")
                    .unwrap();
                codegen
                    .builder
                    .build_store(new_max_len_ptr, combined_length);

                let new_heap_ptr_ptr = codegen
                    .builder
                    .build_struct_gep(alloca, 0, "gepheapptr")
                    .unwrap();
                let malloc = codegen
                    .builder
                    .build_array_malloc(codegen.context.i8_type(), combined_length, "mallocstr")
                    .unwrap();
                codegen.builder.build_store(new_heap_ptr_ptr, malloc);

                // copy the first str into our new str
                // FOR SOME REASON MEMCPY DOESN'T WORK BUT MEMMOVE WORKS,
                // EVEN IF OUR STRINGS AREN'T OVERLAPPING !!
                codegen
                    .builder
                    .build_memmove(
                        malloc,
                        8,
                        left_heap_ptr.into_pointer_value(),
                        8,
                        left_len.into_int_value(),
                    )
                    .unwrap();

                let malloc_offset = unsafe {
                    codegen.builder.build_gep(
                        malloc,
                        &[left_len.into_int_value()],
                        "gepaddstr",
                    )
                };

                // copy the second str
                codegen
                    .builder
                    .build_memmove(
                        malloc_offset,
                        8,
                        right_heap_ptr.into_pointer_value(),
                        8,
                        right_len.into_int_value(),
                    )
                    .unwrap();

                Ok(LNodeResult::Value(LSymbol {
                    value: alloca.into(),
                    symbol_type: SymbolType::Str,
                }))
            }
            _ => Err(CrocoError::add_error(&self.code_pos)),
        }
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
