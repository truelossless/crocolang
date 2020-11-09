#[cfg(feature = "crocoi")]
use crate::crocoi::{INodeResult, ISymTable, ISymbol};

#[cfg(feature = "crocol")]
use {
    crate::crocol::{Codegen, LNodeResult, LSymbol},
    crate::symbol_type::SymbolType,
};

use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::token::{CodePos, LiteralEnum};

/// a node holding a literal value
#[derive(Clone)]
pub struct ConstantNode {
    value: LiteralEnum,
    code_pos: CodePos,
}

impl ConstantNode {
    pub fn new(value: LiteralEnum, code_pos: CodePos) -> Self {
        ConstantNode { value, code_pos }
    }
}

impl AstNode for ConstantNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(ISymbol::Primitive(self.value.clone())))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let constant_symbol = match &self.value {
            LiteralEnum::Bool(b) => LSymbol {
                value: codegen
                    .context
                    .bool_type()
                    .const_int(*b as u64, false)
                    .into(),
                symbol_type: SymbolType::Bool,
            },

            LiteralEnum::Num(n) => LSymbol {
                value: codegen.context.f32_type().const_float(*n as f64).into(),
                symbol_type: SymbolType::Num,
            },

            LiteralEnum::Str(s) => {

                let alloca = codegen.alloc_str(s);

                LSymbol {
                    value: alloca.into(),
                    symbol_type: SymbolType::Str,
                }
            }
        };

        Ok(LNodeResult::Value(constant_symbol))
    }
}
