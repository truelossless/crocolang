use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, LiteralEnum::*},
};

#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::INodeResult, symbol::ISymbol, ISymTable};

#[cfg(feature = "crocol")]
use {
    crate::crocol::{Codegen, LNodeResult, LSymbol},
    inkwell::IntPredicate,
};

#[derive(Clone)]
/// a node used to cast primitives
pub struct AsNode {
    bottom: Option<Box<dyn AstNode>>,
    as_type: SymbolType,
    code_pos: CodePos,
}

impl AsNode {
    pub fn new(as_type: SymbolType, code_pos: CodePos) -> Self {
        AsNode {
            bottom: None,
            as_type,
            code_pos,
        }
    }
}

impl AstNode for AsNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.bottom.is_none() {
            self.bottom = Some(node);
        } else {
            unreachable!();
        }
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let val = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_symbol(&self.code_pos)?;

        // we can only cast primitive together
        let val_primitive = val
            .into_primitive()
            .map_err(|_| CrocoError::cast_non_primitive_error(&self.code_pos))?;

        let casted = match (val_primitive, &self.as_type) {
            // useless cast
            (Bool(_), SymbolType::Bool) | (Str(_), SymbolType::Str) | (Num(_), SymbolType::Num) => {
                return Err(CrocoError::cast_redundant_error(&self.code_pos))
            }

            (Bool(b), SymbolType::Num) => {
                if b {
                    Num(1.)
                } else {
                    Num(0.)
                }
            }
            (Bool(b), SymbolType::Str) => {
                if b {
                    Str("true".to_owned())
                } else {
                    Str("false".to_owned())
                }
            }

            (Num(n), SymbolType::Bool) => {
                if n == 0. {
                    Bool(false)
                } else {
                    Bool(true)
                }
            }
            (Num(n), SymbolType::Str) => Str(n.to_string()),

            (Str(s), SymbolType::Num) => {
                let n = s.parse().map_err(|_| {
                    CrocoError::new(&self.code_pos, "could not parse the str into a num")
                })?;
                Num(n)
            }
            (Str(s), SymbolType::Bool) => {
                if s.is_empty() {
                    Bool(false)
                } else {
                    Bool(true)
                }
            }

            _ => {
                return Err(CrocoError::cast_non_primitive_error(&self.code_pos));
            }
        };

        Ok(INodeResult::Value(ISymbol::Primitive(casted)))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let val = self
            .bottom
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_symbol(codegen, &self.code_pos)?;

        let casted = match (val.symbol_type, &self.as_type) {
            (SymbolType::Bool, SymbolType::Bool)
            | (SymbolType::Str, SymbolType::Str)
            | (SymbolType::Num, SymbolType::Num) => {
                return Err(CrocoError::cast_redundant_error(&self.code_pos))
            }

            (SymbolType::Bool, SymbolType::Num) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    one,
                    "cmpbool",
                );

                LSymbol {
                    value: codegen.builder.build_select(cmp, one, zero, "selectnum"),
                    symbol_type: SymbolType::Num,
                }
            }
            (SymbolType::Bool, SymbolType::Str) => {
                let true_str = codegen.alloc_str("true");
                let false_str = codegen.alloc_str("false");

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    codegen.context.bool_type().const_int(1, false),
                    "cmpbool",
                );

                LSymbol {
                    value: codegen
                        .builder
                        .build_select(cmp, true_str, false_str, "selectstr"),
                    symbol_type: SymbolType::Str,
                }
            }

            (SymbolType::Num, SymbolType::Bool) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let cmp = codegen.builder.build_int_compare(
                    IntPredicate::EQ,
                    val.value.into_int_value(),
                    codegen.context.bool_type().const_int(0, false),
                    "cmpbool",
                );

                LSymbol {
                    value: codegen.builder.build_select(cmp, zero, one, "selectstr"),
                    symbol_type: SymbolType::Bool,
                }
            }
            (SymbolType::Num, SymbolType::Str) => {
                let num_as_str_fn = codegen.module.get_function("_as_num_str").unwrap();
                let str_res = codegen.alloc_str("").into();
                codegen
                    .builder
                    .build_call(num_as_str_fn, &[val.value, str_res], "callcast");

                LSymbol {
                    symbol_type: SymbolType::Str,
                    value: str_res,
                }
            }

            (SymbolType::Str, SymbolType::Num) => {
                let num_as_str_fn = codegen.module.get_function("_as_str_num").unwrap();
                let num_res = codegen.builder.build_call(num_as_str_fn, &[val.value], "callcast")
                    .try_as_basic_value()
                    .left()
                    .unwrap();

                LSymbol {
                    value: num_res,
                    symbol_type: SymbolType::Num
                }
            }
            (SymbolType::Str, SymbolType::Bool) => {
                let zero = codegen.context.bool_type().const_zero();
                let one = codegen.context.bool_type().const_int(1, false);

                let len_ptr = codegen.builder.build_struct_gep(val.value.into_pointer_value(), 1, "geplen").unwrap();
                let len = codegen.builder.build_load(len_ptr, "loadlen");
                
                let cmp = codegen.builder.build_int_compare(IntPredicate::EQ, len.into_int_value(), zero, "cmplen");

                LSymbol {
                    value: codegen.builder.build_select(cmp, zero, one, "selectbool"),
                    symbol_type: SymbolType::Bool
                }
            }

            _ => {
                return Err(CrocoError::cast_non_primitive_error(&self.code_pos));
            }
        };

        Ok(LNodeResult::Value(casted))
    }

    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }
}
